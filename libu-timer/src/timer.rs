use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use libu_derive::*;
use libu_point::*;

const WHEEL_SIZE: usize = 4096;
const TIMER: std::sync::LazyLock<Timer> = std::sync::LazyLock::new(|| Timer::new());

pub fn delay<F>(delay: Duration, f: F) -> TimerHandle
where
  F: FnMut() + Send + 'static,
{
  TIMER.delay(delay, f)
}

pub fn ticker<F>(repeat: Duration, f: F) -> TimerHandle
where
  F: FnMut() + Send + 'static,
{
  TIMER.ticker(repeat, f)
}

type TimerTaskCallback = Box<dyn FnMut() + Send + 'static>;

struct TimerTask {
  remove: bool,
  run: bool,
  /// Absolute tick at which this task should fire next.
  delay: u64,
  /// Repeat interval in ticks. `None` means one-shot.
  repeat: Option<u64>,
  callback: TimerTaskCallback,
}

impl TimerTask {
  fn new<F>(delay: u64, repeat: Option<u64>, f: F) -> Self
  where
    F: FnMut() + Send + 'static,
  {
    Self {
      remove: false,
      run: true,
      delay,
      repeat,
      callback: f.iBox(),
    }
  }
}

#[derive(Clone)]
pub struct TimerHandle(Mrc<TimerTask>);

impl TimerHandle {
  pub fn start(&self) {
    self.0.with_mut(|x| x.run = true);
  }

  pub fn stop(&self) {
    self.0.with_mut(|x| x.run = false);
  }

  pub fn remove(&self) {
    self.0.with_mut(|x| {
      x.run = false;
      x.remove = true;
    });
  }

  /// Returns `true` if the task is currently scheduled to fire.
  ///
  /// A removed task returns `false`. A stopped (but not removed) ticker
  /// also returns `false`; calling `start()` will resume it.
  pub fn is_running(&self) -> bool {
    self.0.with(|x| x.run && !x.remove)
  }

  /// Returns `true` once `remove()` has been called or the callback
  /// panicked (panicking tasks are auto-removed).
  pub fn is_removed(&self) -> bool {
    self.0.with(|x| x.remove)
  }
}

struct TimerWheel {
  tick: u64,
  buckets: Box<[Vec<Arc<Mutex<TimerTask>>>; WHEEL_SIZE]>,
}

impl TimerWheel {
  fn new() -> Self {
    Self {
      tick: 0,
      // Box the array so we don't put a ~64KB stack allocation on
      // every Timer construction.
      buckets: Box::new(std::array::from_fn(|_| Vec::new())),
    }
  }

  fn bucket_of(tick: u64) -> usize {
    (tick % WHEEL_SIZE as u64) as usize
  }

  fn delay<F>(&mut self, delay: u64, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    // Clamp to at least 1 tick. A delay of 0 would target the current
    // bucket, which update() may have already processed this cycle,
    // forcing the task to wait an entire wheel rotation.
    // Use saturating_add so absurdly large delays cannot overflow.
    let fire_at = self.tick.saturating_add(delay.max(1));

    let task = TimerTask::new(fire_at, None, f).iMrc();
    self.buckets[Self::bucket_of(fire_at)].push(task.clone());

    TimerHandle(task)
  }

  fn ticker<F>(&mut self, repeat: u64, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    let repeat = repeat.max(1);
    let fire_at = self.tick.saturating_add(repeat);

    let task = TimerTask::new(fire_at, Some(repeat), f).iMrc();
    self.buckets[Self::bucket_of(fire_at)].push(task.clone());

    TimerHandle(task)
  }

  fn update(&mut self) {
    let current = self.tick;
    let bucket = Self::bucket_of(current);
    let tasks = self.buckets[bucket]
      .extract_if(.., |t| t.with(|x| x.delay == current))
      .collect::<Vec<_>>();

    for task in tasks {
      // Decide whether to re-insert and where to schedule next.
      // Returns Some(next_bucket) if the task should remain in the wheel.
      let next_bucket = task.with_mut(|x| {
        if x.remove {
          return None;
        }

        if x.run {
          // Isolate callback panics so they cannot kill the timer
          // thread. A task that panics is marked for removal to avoid
          // repeated panics on every fire.
          let callback = &mut x.callback;
          let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || callback(),
          ));
          if result.is_err() {
            x.remove = true;
            return None;
          }
        }

        // Tickers stay in the wheel even when stopped, so `start()` can
        // resume them on the next repeat cycle. One-shot tasks that were
        // stopped are dropped (their fire time has passed).
        match x.repeat {
          Some(repeat) => {
            x.delay = current.saturating_add(repeat);
            Some(Self::bucket_of(x.delay))
          }
          None => None,
        }
      });

      if let Some(bucket) = next_bucket {
        self.buckets[bucket].push(task);
      }
    }

    // saturating_add: the wheel becomes effectively frozen at u64::MAX,
    // but that takes ~58 billion years at 100ms ticks.
    self.tick = self.tick.saturating_add(1);
  }
}

pub struct Timer {
  inner: Mrc<TimerWheel>,
  shutdown: Arc<AtomicBool>,
  /// `None` once the worker thread has been joined, or for the global
  /// `LazyLock<Timer>` (which intentionally never shuts down).
  worker: Mutex<Option<JoinHandle<()>>>,
}

impl Timer {
  /// 0.1s
  const TICK: Duration = Duration::from_millis(100);

  pub fn new() -> Self {
    let inner = TimerWheel::new().iMrc();
    let shutdown = Arc::new(AtomicBool::new(false));

    let worker = {
      #[clone(inner, shutdown)]
      let handle = thread::spawn(move || {
        // Schedule against absolute deadlines so update() execution time
        // does not accumulate as drift on top of each sleep.
        let mut next = Instant::now() + Self::TICK;
        while !shutdown.load(Ordering::Acquire) {
          let now = Instant::now();
          if next > now {
            // Sleep in short slices so shutdown is observed promptly
            // even when the next tick is far away.
            let remaining = next - now;
            let slice = remaining.min(Self::TICK);
            thread::sleep(slice);
            continue;
          }
          next += Self::TICK;

          inner.with_mut(|x| x.update());
        }
      });
      handle
    };

    Self {
      inner,
      shutdown,
      worker: Mutex::new(Some(worker)),
    }
  }

  /// Stop the worker thread and wait for it to exit.
  ///
  /// Pending tasks are dropped without firing. After `shutdown`, the
  /// timer no longer ticks; further `delay`/`ticker` calls will still
  /// register tasks but they will never fire.
  pub fn shutdown(&self) {
    self.shutdown.store(true, Ordering::Release);
    if let Some(handle) = self.worker.lock().unwrap_or_else(|e| e.into_inner()).take() {
      let _ = handle.join();
    }
  }

  pub fn delay<F>(&self, delay: Duration, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    let ticks = Self::duration_to_ticks(delay);
    self.inner.with_mut(|x| x.delay(ticks, f))
  }

  pub fn ticker<F>(&self, repeat: Duration, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    let ticks = Self::duration_to_ticks(repeat);
    self.inner.with_mut(|x| x.ticker(ticks, f))
  }

  /// Convert a Duration to a tick count, rounding up so the task never
  /// fires earlier than requested. The wheel itself clamps zero-tick
  /// values to one tick, so a sub-tick duration still schedules.
  fn duration_to_ticks(d: Duration) -> u64 {
    let tick_nanos = Self::TICK.as_nanos();
    let d_nanos = d.as_nanos();
    let ticks = d_nanos.div_ceil(tick_nanos);
    u64::try_from(ticks).unwrap_or(u64::MAX)
  }
}

impl Drop for Timer {
  fn drop(&mut self) {
    self.shutdown();
  }
}
