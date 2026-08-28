//! `Mrc<T>` — thread-shared interior mutability over `Arc<parking_lot::Mutex<T>>`.
//!
//! Call sites normally `use libu::prelude::*;` instead of naming
//! individual traits.
//!
//! | Trait | Method | Requires | Description |
//! |-------|--------|----------|-------------|
//! | [`IntoMrc`] | [`iMrc`](IntoMrc::iMrc) | – | Wrap a value |
//! | [`MrcExt`] | [`at`](MrcExt::at) | `T: Clone` | Clone the value out |
//! | [`MrcExt`] | [`with`](MrcExt::with) / [`with_mut`](MrcExt::with_mut) | – | Blocking closure access |
//! | [`MrcExt`] | [`set`](MrcExt::set) / [`swap`](MrcExt::swap) | – | Overwrite / replace in place |
//! | [`MrcExt`] | [`take`](MrcExt::take) | `T: Default` | Replace with `Default::default()` |
//! | [`MrcExt`] | [`try_with`](MrcExt::try_with) / [`try_with_mut`](MrcExt::try_with_mut) | – | Non-blocking closure access |
//! | [`MrcExt`] | [`ptr_eq`](MrcExt::ptr_eq) | – | Same allocation? |
//! | [`MrcExt`] | [`val_eq`](MrcExt::val_eq) | `T: PartialEq` | Compare inner values |

use parking_lot::Mutex;

use std::sync::Arc;

pub type Mrc<T> = Arc<Mutex<T>>;

#[extend::ext(pub, name = IntoMrc)]
impl<T> T {
  /// Wrap a value in an [`Mrc`].
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let m = 1.iMrc();
  /// assert_eq!(m.at(), 1);
  /// ```
  fn iMrc(self) -> Mrc<T> {
    Arc::new(Mutex::new(self))
  }
}

#[extend::ext(pub, name = MrcExt)]
impl<T> Mrc<T> {
  /// Clone the value out, producing a snapshot independent of later
  /// mutation.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let m = 0.iMrc();
  /// let snap = m.at();
  ///
  /// m.set(99);
  /// assert_eq!(snap, 0);
  /// assert_eq!(m.at(), 99);
  /// ```
  fn at(&self) -> T
  where
    T: Clone,
  {
    self.lock().clone()
  }

  /// Run `f` with shared access, blocking until the mutex is free.
  ///
  /// This is the read half of the locked-closure pattern: nothing else
  /// can mutate while `f` runs, and `parking_lot` mutexes cannot poison,
  /// so there is no lock error to handle.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let m = vec![1, 2, 3].iMrc();
  /// assert_eq!(m.with(|v| v.len()), 3);
  /// ```
  fn with<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&T) -> R,
  {
    f(&*self.lock())
  }

  /// Run `f` with exclusive access, blocking until the mutex is free.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let m = vec![1, 2, 3].iMrc();
  /// m.with_mut(|v| v.push(4));
  /// assert_eq!(m.at(), [1, 2, 3, 4]);
  /// ```
  fn with_mut<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&mut T) -> R,
  {
    f(&mut *self.lock())
  }

  /// Overwrite the value in place.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let m = 1.iMrc();
  /// m.set(2);
  /// assert_eq!(m.at(), 2);
  /// ```
  fn set(&self, v: T) {
    *self.lock() = v;
  }

  /// Replace the value and return the old one.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let m = "old".iMrc();
  /// assert_eq!(m.swap("new"), "old");
  /// assert_eq!(m.at(), "new");
  /// ```
  fn swap(&self, v: T) -> T {
    std::mem::replace(&mut *self.lock(), v)
  }

  /// Replace the value with `Default::default()` and return the old one.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let m = 42.iMrc();
  /// assert_eq!(m.take(), 42);
  /// assert_eq!(m.take(), 0);
  /// ```
  fn take(&self) -> T
  where
    T: Default,
  {
    std::mem::take(&mut *self.lock())
  }

  /// Run `f` with shared access without blocking.
  ///
  /// Returns `None` if the mutex is currently locked, where
  /// [`MrcExt::with`] would block until it becomes available.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let m = 1.iMrc();
  /// assert_eq!(m.try_with(|x| x + 1), Some(2));
  /// ```
  fn try_with<F, R>(&self, f: F) -> Option<R>
  where
    F: FnOnce(&T) -> R,
  {
    Some(f(&*self.try_lock()?))
  }

  /// Run `f` with exclusive access without blocking.
  ///
  /// Returns `None` if the mutex is currently locked, where
  /// [`MrcExt::with_mut`] would block until it becomes available.
  fn try_with_mut<F, R>(&self, f: F) -> Option<R>
  where
    F: FnOnce(&mut T) -> R,
  {
    Some(f(&mut *self.try_lock()?))
  }

  /// Returns `true` if both handles share the same allocation.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let m = 1.iMrc();
  /// assert!(m.ptr_eq(&m.clone()));
  /// assert!(!m.ptr_eq(&1.iMrc()));
  /// ```
  fn ptr_eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(self, other)
  }

  /// Compare the values behind two handles.
  ///
  /// Aliased handles (`ptr_eq`) are compared through a single lock so the
  /// call stays reentrancy-safe. Distinct handles are locked in raw-pointer
  /// order so concurrent `val_eq` calls in opposite directions cannot
  /// deadlock (ABBA).
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoMrc, MrcExt};
  ///
  /// let a = 1.iMrc();
  /// assert!(a.val_eq(&a.clone()));
  /// assert!(!a.val_eq(&2.iMrc()));
  /// ```
  fn val_eq(&self, other: &Self) -> bool
  where
    T: PartialEq,
  {
    if Arc::ptr_eq(self, other) {
      let this = self.lock();
      let v = &*this;
      return *v == *v;
    }

    let (one, two) = if Arc::as_ptr(self) < Arc::as_ptr(other) {
      (self.lock(), other.lock())
    } else {
      (other.lock(), self.lock())
    };

    *one == *two
  }
}
