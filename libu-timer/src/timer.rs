use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const WHEEL_SIZE: usize = 4096;
const TIMER: std::sync::LazyLock<Timer> = std::sync::LazyLock::new(|| Timer::new());

pub fn delay(delay: usize, f: impl FnMut() + Send + 'static) -> TimerHandle {
  TIMER.delay(delay, f)
}

pub fn ticker(repeat: usize, f: impl FnMut() + Send + 'static) -> TimerHandle {
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
  fn new(delay: usize, repeat: Option<usize>, f: impl FnMut() + Send + 'static) -> Self {
    Self {
      remove: false,
      run: true,
      delay: delay,
      repeat: repeat,
      callback: Box::new(f),
    }
  }
}

pub struct TimerHandle {
  inner: Arc<Mutex<TimerTask>>,
}

impl TimerHandle {
  pub fn start(&mut self) {
    self.inner.try_lock().unwrap().run = true;
  }

  pub fn stop(&mut self) {
    self.inner.try_lock().unwrap().run = false;
  }

  pub fn remove(&mut self) {
    self.inner.try_lock().unwrap().run = false;
    self.inner.try_lock().unwrap().remove = true;
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

  fn delay(&mut self, delay: usize, f: impl FnMut() + Send + 'static) -> TimerHandle {
    let delay = self.tick + delay;

    let task = Arc::new(Mutex::new(TimerTask::new(delay, None, f)));
    self.buckets[delay % WHEEL_SIZE].push_back(task.clone());

    TimerHandle { inner: task }
  }

  fn ticker(&mut self, repeat: usize, f: impl FnMut() + Send + 'static) -> TimerHandle {
    let delay = self.tick + repeat;

    let task = Arc::new(Mutex::new(TimerTask::new(delay, Some(repeat), f)));
    self.buckets[delay % WHEEL_SIZE].push_back(task.clone());

    TimerHandle { inner: task }
  }

  fn update(&mut self) {
    let tasks = self.buckets[self.tick % WHEEL_SIZE]
      .extract_if(|t| t.try_lock().unwrap().delay == self.tick)
      .collect::<Vec<_>>();

    for task in tasks {
      let mut task_mut = task.lock().unwrap();
      if task_mut.run {
        (task_mut.callback)();
      }

      if !task_mut.remove
        && let Some(repeat) = task_mut.repeat
      {
        task_mut.delay = self.tick + repeat;
      }

      self.buckets[task_mut.delay % WHEEL_SIZE].push_back(task.clone());
    }

    self.tick += 1;
  }
}

pub struct Timer {
  inner: Arc<Mutex<TimerWheel>>,
}

impl Timer {
  pub fn new() -> Self {
    let this = Self {
      inner: Arc::new(Mutex::new(TimerWheel::new())),
    };

    let timer = Arc::clone(&this.inner);
    const TICK: Duration = Duration::from_millis(100);
    thread::spawn(move || {
      loop {
        thread::sleep(TICK);

        timer.try_lock().unwrap().update();
      }
    });

    this
  }

  pub fn delay(&self, delay: usize, f: impl FnMut() + Send + 'static) -> TimerHandle {
    self.inner.lock().unwrap().delay(delay, f)
  }

  pub fn ticker(&self, repeat: usize, f: impl FnMut() + Send + 'static) -> TimerHandle {
    self.inner.lock().unwrap().ticker(repeat, f)
  }
}
