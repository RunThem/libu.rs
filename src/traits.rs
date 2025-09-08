use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use extend::ext;

#[ext(pub, name = Pick)]
impl<O> bool {
  #[inline]
  fn pick(self, te: O, fe: O) -> O {
    if self { te } else { fe }
  }
}

#[ext(pub, name = Bzero)]
impl<A: Default> A {
  #[inline]
  fn bzero(&mut self) {
    *self = Default::default()
  }
}

#[ext(pub, name = Void)]
impl<B: Sized> B {
  #[inline]
  fn void(self) {}
}

#[ext(pub, name = ToDur)]
impl str {
  #[inline]
  fn to_dur(&self) -> Duration {
    let len = self.find(|c: char| !c.is_ascii_digit()).unwrap();
    let (num, unit) = self.split_at(len);
    let num = num.parse::<u64>().unwrap();

    match unit {
      "ns" => Duration::from_nanos(num),
      "us" => Duration::from_micros(num),
      "ms" => Duration::from_millis(num),
      "s" => Duration::from_secs(num),
      "m" => Duration::from_secs(num * 60),

      _ => panic!("unsupported time units."),
    }
  }
}

#[ext(pub, name = RemoveIf)]
impl<T, F: Fn(&T) -> bool> Vec<T> {
  fn remove_if(&mut self, predicate: F) -> Self {
    let mut i = 0;
    let mut r = Self::with_capacity(self.len());

    while i < self.len() {
      crate::when! {
        predicate(&self[i]) => r.push(self.remove(i)),
        @ => i+=1,
      }
    }

    r
  }
}
