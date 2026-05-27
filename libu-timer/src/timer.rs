//! Single-level timing wheel with a 100ms tick.
//!
//! # Resolution and range
//!
//! - Tick resolution: 100ms. Durations are rounded up to the next tick.
//! - Wheel size: 4096 buckets, so a task targeting `tick + N` lands in
//!   `buckets[(tick + N) % 4096]`.
//! - There is no upper bound on `Duration`. Tasks with delays larger
//!   than one wheel rotation (~409 seconds) still fire correctly: each
//!   task carries its absolute target tick, and dispatch only fires
//!   tasks whose target tick matches the current tick exactly.
//!
//! # Performance characteristics
//!
//! - Registration is O(1) plus a per-bucket mutex acquisition.
//! - Dispatch scans only the current tick's bucket, not the whole wheel.
//! - However, long-delay tasks remain pinned to one bucket for their
//!   entire lifetime. Each wheel rotation (~409s) the dispatcher walks
//!   that bucket and checks every task. Workloads with very large
//!   numbers of long-delay tasks (e.g. hundreds of thousands of tasks
//!   each ~hours out) will see this scan cost grow.
//!
//!   A hierarchical timing wheel would amortize this by cascading
//!   long-delay tasks down through coarser levels. It is not
//!   implemented here.
//!
//! # Concurrency
//!
//! - `tick` is an `AtomicU64` written only by the worker thread.
//! - Each of the 4096 buckets has its own `Mutex`. Registration and
//!   dispatch only contend when they target the same bucket.
//! - Callbacks run with no bucket lock held, so a slow callback does
//!   not block concurrent registrations.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
  /// Monotonically advances each `update()`. Written only by the
  /// worker thread; read by `delay`/`ticker` to compute target buckets.
  tick: AtomicU64,
  /// One mutex per bucket. Registration and dispatch only contend when
  /// they target the same bucket.
  buckets: Box<[Mutex<Vec<Arc<Mutex<TimerTask>>>>; WHEEL_SIZE]>,
}

impl TimerWheel {
  fn new() -> Self {
    Self {
      tick: AtomicU64::new(0),
      // Box the array so we don't put a ~64KB stack allocation on
      // every Timer construction.
      buckets: Box::new(std::array::from_fn(|_| Mutex::new(Vec::new()))),
    }
  }

  fn bucket_of(tick: u64) -> usize {
    (tick % WHEEL_SIZE as u64) as usize
  }

  fn lock_bucket(&self, bucket: usize) -> std::sync::MutexGuard<'_, Vec<Arc<Mutex<TimerTask>>>> {
    self.buckets[bucket]
      .lock()
      .unwrap_or_else(|e| e.into_inner())
  }

  fn delay<F>(&self, delay: u64, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    // Clamp to at least 1 tick. A delay of 0 would target the current
    // bucket, which update() may have already processed this cycle,
    // forcing the task to wait an entire wheel rotation.
    // Use saturating_add so absurdly large delays cannot overflow.
    let fire_at = self.tick.load(Ordering::Acquire).saturating_add(delay.max(1));

    let task = TimerTask::new(fire_at, None, f).iMrc();
    self.lock_bucket(Self::bucket_of(fire_at)).push(task.clone());

    TimerHandle(task)
  }

  fn ticker<F>(&self, repeat: u64, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    let repeat = repeat.max(1);
    let fire_at = self.tick.load(Ordering::Acquire).saturating_add(repeat);

    let task = TimerTask::new(fire_at, Some(repeat), f).iMrc();
    self.lock_bucket(Self::bucket_of(fire_at)).push(task.clone());

    TimerHandle(task)
  }

  fn update(&self) {
    let current = self.tick.load(Ordering::Acquire);
    let bucket = Self::bucket_of(current);

    // Only hold the bucket lock long enough to extract due tasks, then
    // release so callbacks (which may take arbitrary time) don't block
    // concurrent delay/ticker registrations.
    let tasks: Vec<Arc<Mutex<TimerTask>>> = {
      let mut guard = self.lock_bucket(bucket);
      guard.extract_if(.., |t| t.with(|x| x.delay == current)).collect()
    };

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
        self.lock_bucket(bucket).push(task);
      }
    }

    // saturating_add: the wheel becomes effectively frozen at u64::MAX,
    // but that takes ~58 billion years at 100ms ticks. The worker
    // thread is the only writer, so a plain load/store is sufficient.
    let next = current.saturating_add(1);
    self.tick.store(next, Ordering::Release);
  }
}

/// Shared timer handle. Cloning is cheap (a single `Arc` bump) and
/// safe to send across threads. The worker thread is stopped and joined
/// when the **last** clone is dropped.
#[derive(Clone)]
pub struct Timer(Arc<TimerInner>);

struct TimerInner {
  wheel: Arc<TimerWheel>,
  shutdown: Arc<AtomicBool>,
  /// `None` once the worker thread has been joined.
  worker: Mutex<Option<JoinHandle<()>>>,
}

impl Timer {
  /// 0.1s
  const TICK: Duration = Duration::from_millis(100);

  pub fn new() -> Self {
    let wheel = Arc::new(TimerWheel::new());
    let shutdown = Arc::new(AtomicBool::new(false));

    let worker = {
      #[clone(wheel, shutdown)]
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

          wheel.update();
        }
      });
      handle
    };

    Self(Arc::new(TimerInner {
      wheel,
      shutdown,
      worker: Mutex::new(Some(worker)),
    }))
  }

  /// Stop the worker thread and wait for it to exit.
  ///
  /// Pending tasks are dropped without firing. After `shutdown`, the
  /// timer no longer ticks; further `delay`/`ticker` calls will still
  /// register tasks but they will never fire. Safe to call from any
  /// clone and to call multiple times.
  pub fn shutdown(&self) {
    self.0.shutdown.store(true, Ordering::Release);
    if let Some(handle) = self
      .0
      .worker
      .lock()
      .unwrap_or_else(|e| e.into_inner())
      .take()
    {
      let _ = handle.join();
    }
  }

  pub fn delay<F>(&self, delay: Duration, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    let ticks = Self::duration_to_ticks(delay);
    self.0.wheel.delay(ticks, f)
  }

  pub fn ticker<F>(&self, repeat: Duration, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    let ticks = Self::duration_to_ticks(repeat);
    self.0.wheel.ticker(ticks, f)
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

impl Drop for TimerInner {
  fn drop(&mut self) {
    // Only reached when the last Timer clone is dropped, since Timer
    // holds Arc<TimerInner>. Stop the worker and join it.
    self.shutdown.store(true, Ordering::Release);
    if let Some(handle) = self.worker.lock().unwrap_or_else(|e| e.into_inner()).take() {
      let _ = handle.join();
    }
  }
}
