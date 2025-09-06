use std::collections::LinkedList;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use libu_point::Mrc;

const WHEEL_SIZE: usize = 4096;
const TIMER: std::sync::LazyLock<Timer> = std::sync::LazyLock::new(|| Timer::new());

pub fn delay(delay: usize, f: impl FnMut() + Send + 'static) -> TimerTask {
  TIMER.delay(delay, f)
}

pub fn ticker(repeat: usize, f: impl FnMut() + Send + 'static) -> TimerTask {
  TIMER.ticker(repeat, f)
}

type TimerTaskCallback = Box<dyn FnMut() + Send + 'static>;

struct TimerTaskInner {
  remove: bool,
  run: bool,
  delay: usize,
  repeat: Option<usize>,
  callback: TimerTaskCallback,
}

impl TimerTaskInner {
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

pub struct TimerTask {
  inner: Mrc<TimerTaskInner>,
}

unsafe impl Sync for TimerTask {}
unsafe impl Send for TimerTask {}

impl TimerTask {
  pub fn start(&mut self) {
    self.inner.run = true;
  }

  pub fn stop(&mut self) {
    self.inner.run = false;
  }

  pub fn remove(&mut self) {
    self.inner.run = false;
    self.inner.remove = true;
  }
}

struct TimerWheel {
  tick: usize,
  buckets: [LinkedList<TimerTask>; WHEEL_SIZE],
}

impl TimerWheel {
  fn new() -> Self {
    Self {
      tick: 0,
      buckets: std::array::from_fn(|_| LinkedList::new()),
    }
  }

  fn delay(&mut self, delay: usize, f: impl FnMut() + Send + 'static) -> TimerTask {
    let delay = self.tick + delay;

    let task = Mrc::new(TimerTaskInner::new(delay, None, f));

    self.buckets[delay % WHEEL_SIZE].push_back(TimerTask {
      inner: task.clone(),
    });

    TimerTask {
      inner: task.clone(),
    }
  }

  fn ticker(&mut self, repeat: usize, f: impl FnMut() + Send + 'static) -> TimerTask {
    let delay = self.tick + repeat;

    let task = Mrc::new(TimerTaskInner::new(delay, Some(repeat), f));

    self.buckets[delay % WHEEL_SIZE].push_back(TimerTask {
      inner: task.clone(),
    });

    TimerTask {
      inner: task.clone(),
    }
  }

  fn update(&mut self) {
    let buckets = &mut self.buckets[self.tick % WHEEL_SIZE];

    let mut ready = LinkedList::new();
    let mut cursor = buckets.cursor_front_mut();
    while let Some(task) = cursor.current() {
      if task.inner.delay == self.tick {
        ready.push_back(cursor.remove_current().unwrap());
      } else {
        cursor.move_next();
      }
    }

    for mut task in ready {
      if task.inner.run {
        (task.inner.callback)();
      }

      if !task.inner.remove
        && let Some(repeat) = task.inner.repeat
      {
        task.inner.delay = self.tick + repeat;

        self.buckets[task.inner.delay % WHEEL_SIZE].push_back(task);
      }
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

  pub fn delay(&self, delay: usize, f: impl FnMut() + Send + 'static) -> TimerTask {
    self.inner.lock().unwrap().delay(delay, f)
  }

  pub fn ticker(&self, repeat: usize, f: impl FnMut() + Send + 'static) -> TimerTask {
    self.inner.lock().unwrap().ticker(repeat, f)
  }
}
