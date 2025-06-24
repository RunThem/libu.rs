use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// [`Mrc`]
pub type Mrc<T> = Rc<RefCell<T>>;

/// [`Point`]
pub trait Point {
  #[allow(non_snake_case)]
  fn iBox(self) -> Box<Self>;

  #[allow(non_snake_case)]
  fn iArc(self) -> Arc<Self>;

  #[allow(non_snake_case)]
  fn iMrc(self) -> Mrc<Self>;

  #[allow(non_snake_case)]
  fn iMut(self) -> Mutex<Self>;
}

impl<T> Point for T {
  #[inline]
  #[allow(non_snake_case)]
  fn iBox(self) -> Box<Self> {
    Box::new(self)
  }

  #[inline]
  #[allow(non_snake_case)]
  fn iArc(self) -> Arc<Self> {
    Arc::new(self)
  }

  #[inline]
  #[allow(non_snake_case)]
  fn iMrc(self) -> Mrc<Self> {
    Rc::new(RefCell::new(self))
  }

  #[inline]
  #[allow(non_snake_case)]
  fn iMut(self) -> Mutex<Self> {
    Mutex::new(self)
  }
}

/// [`Pick`]
pub trait Pick<O> {
  #[allow(non_snake_case)]
  #[allow(unused)]
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
  fn default(&mut self);
}

impl<T> Bzero for T
where
  T: Default,
{
  fn default(&mut self) {
    *self = Default::default()
  }
}

/// [`Discard`]
pub trait Discard {
  fn discard(self);
}

impl<T> Discard for T {
  fn discard(self) {}
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
pub trait RemoveIf<F>
where
  F: Fn(&Self::Item) -> bool,
{
  type Item;

  fn remove_if(&mut self, predicate: F) -> Self;
}

impl<T, F> RemoveIf<F> for Vec<T>
where
  F: Fn(&T) -> bool,
{
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
