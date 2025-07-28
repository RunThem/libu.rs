use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use libu_urc::Urc;

/// [`Mrc`]
pub type Mrc<T> = Rc<RefCell<T>>;

/// [`Point`]
pub trait Point {
  fn iBox(self) -> Box<Self>;
  fn iMrc(self) -> Mrc<Self>;
  fn iArc(self) -> Arc<Self>;
  fn iMut(self) -> Mutex<Self>;

  fn iUrc(self) -> Urc<Self>
  where
    Self: Sized;
}

impl<T: Sized> Point for T {
  #[inline]
  fn iBox(self) -> Box<Self> {
    Box::new(self)
  }

  #[inline]
  fn iArc(self) -> Arc<Self> {
    Arc::new(self)
  }

  #[inline]
  fn iMrc(self) -> Mrc<Self> {
    Rc::new(RefCell::new(self))
  }

  #[inline]
  fn iMut(self) -> Mutex<Self> {
    Mutex::new(self)
  }

  #[inline]
  fn iUrc(self) -> Urc<Self> {
    Urc::new(self)
  }
}

/// [`Pick`]
pub trait Pick<O> {
  fn pick(self, te: O, fe: O) -> O;
}

impl<O> Pick<O> for bool {
  #[inline]
  fn pick(self, te: O, fe: O) -> O {
    if self { te } else { fe }
  }
}

/// [`Bzero`]
pub trait Bzero {
  #[allow(unused)]
  fn bzero(&mut self);
}

impl<T: Default> Bzero for T {
  fn bzero(&mut self) {
    *self = Default::default()
  }
}

/// [`Void`]
pub trait Void {
  fn void(self);
}

impl<T: Sized> Void for T {
  fn void(self) {}
}

/// [`ToDuration`]
pub trait ToDuration {
  fn to_dur(&self) -> Duration;
}

impl ToDuration for str {
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

/// [`RemoveIf`]
pub trait RemoveIf<F: Fn(&Self::Item) -> bool> {
  type Item;

  fn remove_if(&mut self, predicate: F) -> Self;
}

impl<T, F: Fn(&T) -> bool> RemoveIf<F> for Vec<T> {
  type Item = T;

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
