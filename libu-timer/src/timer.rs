use std::collections::LinkedList;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const WHEEL_SIZE: usize = 4096;
const TIMER: std::sync::LazyLock<Timer> = std::sync::LazyLock::new(|| Timer::new());

pub fn delay(delay: usize, f: impl Fn() + Send + 'static) {
  TIMER.delay(delay, f);
}

pub fn ticker(repeat: usize, f: impl Fn() + Send + 'static) {
  TIMER.ticker(repeat, f);
}

type TimerCallback = Box<dyn Fn() + Send + 'static>;

struct TimerTask {
  delay: usize,
  repeat: Option<usize>,
  callback: TimerCallback,
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

  fn delay(&mut self, delay: usize, f: impl Fn() + Send + 'static) {
    let delay = self.tick + delay;

    self.buckets[delay % WHEEL_SIZE].push_back(TimerTask {
      delay,
      repeat: None,
      callback: Box::new(f),
    });
  }

  fn ticker(&mut self, repeat: usize, f: impl Fn() + Send + 'static) {
    let delay = self.tick + repeat;

    self.buckets[delay % WHEEL_SIZE].push_back(TimerTask {
      delay,
      repeat: Some(repeat),
      callback: Box::new(f),
    });
  }

  fn update(&mut self) {
    let buckets = &mut self.buckets[self.tick % WHEEL_SIZE];

    let mut ready = LinkedList::new();
    let mut cursor = buckets.cursor_front_mut();
    while let Some(task) = cursor.current() {
      if task.delay == self.tick {
        ready.push_back(cursor.remove_current().unwrap());
      } else {
        cursor.move_next();
      }
    }

    for task in ready {
      (task.callback)();
      if let Some(repeat) = task.repeat {
        self.ticker(repeat, task.callback);
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

  pub fn delay(&self, delay: usize, f: impl Fn() + Send + 'static) {
    self.inner.lock().unwrap().delay(delay, f);
  }

  pub fn ticker(&self, repeat: usize, f: impl Fn() + Send + 'static) {
    self.inner.lock().unwrap().ticker(repeat, f);
  }
}
