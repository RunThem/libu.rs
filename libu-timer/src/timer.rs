use std::collections::LinkedList;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use libu_derive::*;
use libu_point::*;

const WHEEL_SIZE: usize = 4096;
const TIMER: std::sync::LazyLock<Timer> = std::sync::LazyLock::new(|| Timer::new());

pub fn delay<F>(delay: usize, f: F) -> TimerHandle
where
  F: FnMut() + Send + 'static,
{
  TIMER.delay(delay, f)
}

pub fn ticker<F>(repeat: usize, f: F) -> TimerHandle
where
  F: FnMut() + Send + 'static,
{
  TIMER.ticker(repeat, f)
}

type TimerTaskCallback = Box<dyn FnMut() + Send + 'static>;

struct TimerTask {
  remove: bool,
  run: bool,
  delay: usize,
  repeat: Option<usize>,
  callback: TimerTaskCallback,
}

unsafe impl Sync for TimerTask {}
unsafe impl Send for TimerTask {}

impl TimerTask {
  fn new<F>(delay: usize, repeat: Option<usize>, f: F) -> Self
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

pub struct TimerHandle(Mrc<TimerTask>);

impl TimerHandle {
  pub fn start(&mut self) {
    self.0.with_mut(|x| x.run = true);
  }

  pub fn stop(&mut self) {
    self.0.with_mut(|x| x.run = false);
  }

  pub fn remove(&mut self) {
    self.0.with_mut(|x| {
      x.run = false;
      x.remove = true;
    });
  }
}

struct TimerWheel {
  tick: usize,
  buckets: [LinkedList<Arc<Mutex<TimerTask>>>; WHEEL_SIZE],
}

impl TimerWheel {
  fn new() -> Self {
    Self {
      tick: 0,
      buckets: std::array::from_fn(|_| LinkedList::new()),
    }
  }

  fn delay<F>(&mut self, delay: usize, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    let delay = self.tick + delay;

    let task = TimerTask::new(delay, None, f).iMrc();
    self.buckets[delay % WHEEL_SIZE].push_back(task.clone());

    TimerHandle(task)
  }

  fn ticker<F>(&mut self, repeat: usize, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    let delay = self.tick + repeat;

    let task = TimerTask::new(delay, Some(repeat), f).iMrc();
    self.buckets[delay % WHEEL_SIZE].push_back(task.clone());

    TimerHandle(task)
  }

  fn update(&mut self) {
    let tasks = self.buckets[self.tick % WHEEL_SIZE]
      .extract_if(|t| t.with(|x| x.delay == self.tick))
      .collect::<Vec<_>>();

    for task in tasks {
      task.with_mut(|x| {
        if x.run {
          (x.callback)();

          if let Some(repeat) = x.repeat {
            if !x.remove {
              x.delay = self.tick + repeat;
            }
          };

          self.buckets[x.delay % WHEEL_SIZE].push_back(task.clone());
        }
      })
    }

    self.tick += 1;
  }
}

pub struct Timer(Mrc<TimerWheel>);

impl Timer {
  /// 0.1s
  const TICK: Duration = Duration::from_millis(100);

  pub fn new() -> Self {
    let inner = TimerWheel::new().iMrc();

    #[clone(inner)]
    thread::spawn(move || {
      loop {
        thread::sleep(Self::TICK);

        inner.with_mut(|x| x.update());
      }
    });

    Self(inner)
  }

  pub fn delay<F>(&self, delay: usize, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    self.0.with_mut(|x| x.delay(delay, f))
  }

  pub fn ticker<F>(&self, repeat: usize, f: F) -> TimerHandle
  where
    F: FnMut() + Send + 'static,
  {
    self.0.with_mut(|x| x.ticker(repeat, f))
  }
}
