use std::cell::{Cell, UnsafeCell};
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

struct MrcInner<T: Sized> {
  refcount: Cell<usize>,
  value: UnsafeCell<T>,
}

pub struct Mrc<T: Sized> {
  inner: NonNull<MrcInner<T>>,
}

impl<T: Sized> Mrc<T> {
  pub fn new(value: T) -> Self {
    let inner = Box::new(MrcInner {
      refcount: Cell::new(1),
      value: UnsafeCell::new(value),
    });

    Self {
      inner: NonNull::new(Box::into_raw(inner)).unwrap(),
    }
  }

  #[inline]
  fn refcount(&self) -> &Cell<usize> {
    unsafe { &self.inner.as_ref().refcount }
  }

  #[inline]
  fn refcount_inc(&self) {
    self.refcount().set(self.refcount().get() + 1)
  }

  #[inline]
  fn refcount_dec(&self) {
    self.refcount().set(self.refcount().get() - 1)
  }
}

impl<T: Sized + Debug> Debug for Mrc<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    unsafe { write!(f, "{:#?}", *self.inner.as_ref().value.get()) }
  }
}

impl<T: Sized + Display> Display for Mrc<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    unsafe { write!(f, "{}", *self.inner.as_ref().value.get()) }
  }
}

impl<T: Sized> Clone for Mrc<T> {
  fn clone(&self) -> Self {
    self.refcount_inc();

    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<T: Sized> Deref for Mrc<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.inner.as_ref().value.get() }
  }
}

impl<T: Sized> DerefMut for Mrc<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.inner.as_ref().value.get() }
  }
}

impl<T: Sized> Drop for Mrc<T> {
  fn drop(&mut self) {
    if self.refcount().get() == 1 {
      unsafe { NonNull::drop_in_place(self.inner) };
    } else {
      self.refcount_dec();
    }
  }
}
