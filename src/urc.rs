use std::cell::{Cell, UnsafeCell};
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

struct UrcInner<T: Sized> {
  refcount: Cell<usize>,
  value: UnsafeCell<T>,
}

pub struct Urc<T: Sized> {
  inner: NonNull<UrcInner<T>>,
}

impl<T: Sized> Urc<T> {
  pub fn new(value: T) -> Self {
    let inner = Box::new(UrcInner {
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

impl<T: Sized + Debug> Debug for Urc<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    unsafe { writeln!(f, "{:#?}", *self.inner.as_ref().value.get()) }
  }
}

impl<T: Sized + Display> Display for Urc<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    unsafe { writeln!(f, "{}", *self.inner.as_ref().value.get()) }
  }
}

impl<T: Sized> Clone for Urc<T> {
  fn clone(&self) -> Self {
    self.refcount_inc();

    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<T: Sized> Deref for Urc<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.inner.as_ref().value.get() }
  }
}

impl<T: Sized> DerefMut for Urc<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.inner.as_ref().value.get() }
  }
}

impl<T: Sized> Drop for Urc<T> {
  fn drop(&mut self) {
    if self.refcount().get() == 1 {
      unsafe { NonNull::drop_in_place(self.inner) };
    } else {
      self.refcount_dec();
    }
  }
}
