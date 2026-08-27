//! `Urc<T>` — single-threaded interior mutability over `Rc<RefCell<T>>`.
//!
//! Call sites normally `use libu::prelude::*;` instead of naming
//! individual traits.
//!
//! | Trait | Method | Requires | Description |
//! |-------|--------|----------|-------------|
//! | [`IntoUrc`] | [`iUrc`](IntoUrc::iUrc) | – | Wrap a value |
//! | [`UrcExt`] | [`at`](UrcExt::at) | `T: Clone` | Clone the value out |
//! | [`UrcExt`] | [`with`](UrcExt::with) / [`with_mut`](UrcExt::with_mut) | – | Panicking closure access |
//! | [`UrcExt`] | [`set`](UrcExt::set) / [`swap`](UrcExt::swap) | – | Overwrite / replace in place |
//! | [`UrcExt`] | [`take`](UrcExt::take) | `T: Default` | Replace with `Default::default()` |
//! | [`UrcExt`] | [`try_with`](UrcExt::try_with) / [`try_with_mut`](UrcExt::try_with_mut) | – | Non-panicking closure access |
//! | [`UrcExt`] | [`ptr_eq`](UrcExt::ptr_eq) | – | Same allocation? |
//! | [`UrcExt`] | [`val_eq`](UrcExt::val_eq) | `T: PartialEq` | Compare inner values |

use std::cell::RefCell;
use std::rc::Rc;

pub type Urc<T> = Rc<RefCell<T>>;

#[extend::ext(pub, name = IntoUrc)]
impl<T> T {
  /// Wrap a value in a [`Urc`].
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let u = 1.iUrc();
  /// assert!(u.ptr_eq(&u.clone()));
  /// ```
  fn iUrc(self) -> Urc<T> {
    Rc::new(RefCell::new(self))
  }
}

#[extend::ext(pub, name = UrcExt)]
impl<T> Urc<T> {
  /// Clone the value out, producing a snapshot independent of later
  /// mutation.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let u = 0.iUrc();
  /// let snap = u.at();
  ///
  /// u.set(99);
  /// assert_eq!(snap, 0);
  /// assert_eq!(u.at(), 99);
  /// ```
  fn at(&self) -> T
  where
    T: Clone,
  {
    self.borrow().clone()
  }

  /// Run `f` with shared access.
  ///
  /// Panics if another clone currently holds a **mutable** borrow, the
  /// same way a bare `borrow()` would; use [`UrcExt::try_with`] for the
  /// non-panicking variant.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let u = vec![1, 2, 3].iUrc();
  /// assert_eq!(u.with(|v| v.len()), 3);
  /// ```
  fn with<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&T) -> R,
  {
    f(&*self.borrow())
  }

  /// Run `f` with exclusive access.
  ///
  /// Panics if another clone currently holds any borrow, the same way a
  /// bare `borrow_mut()` would; use [`UrcExt::try_with_mut`] for the
  /// non-panicking variant.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let u = vec![1, 2, 3].iUrc();
  /// u.with_mut(|v| v.push(4));
  /// assert_eq!(u.at(), [1, 2, 3, 4]);
  /// ```
  fn with_mut<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&mut T) -> R,
  {
    f(&mut *self.borrow_mut())
  }

  /// Overwrite the value in place.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let u = 1.iUrc();
  /// u.set(2);
  /// assert_eq!(u.at(), 2);
  /// ```
  fn set(&self, v: T) {
    *self.borrow_mut() = v;
  }

  /// Replace the value and return the old one.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let u = "old".iUrc();
  /// assert_eq!(u.swap("new"), "old");
  /// assert_eq!(u.at(), "new");
  /// ```
  fn swap(&self, v: T) -> T {
    std::mem::replace(&mut *self.borrow_mut(), v)
  }

  /// Replace the value with `Default::default()` and return the old one.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let u = 42.iUrc();
  /// assert_eq!(u.take(), 42);
  /// assert_eq!(u.take(), 0);
  /// ```
  fn take(&self) -> T
  where
    T: Default,
  {
    std::mem::take(&mut *self.borrow_mut())
  }

  /// Run `f` with shared access without panicking.
  ///
  /// Shared (`&`) borrows nest freely — this only returns `None` when
  /// another clone currently holds a **mutable** borrow, which would make
  /// plain `borrow()` panic with a `BorrowError`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let u = 1.iUrc();
  ///
  /// // An outstanding shared guard blocks exclusive access...
  /// let guard = u.borrow();
  /// assert_eq!(u.try_with_mut(|x| *x += 1), None);
  /// // ...while other shared accesses still nest.
  /// assert_eq!(u.try_with(|x| x + 1), Some(2));
  /// ```
  fn try_with<F, R>(&self, f: F) -> Option<R>
  where
    F: FnOnce(&T) -> R,
  {
    Some(f(&*self.try_borrow().ok()?))
  }

  /// Run `f` with exclusive access without panicking.
  ///
  /// Returns `None` if another clone currently holds any borrow of this
  /// value, where [`UrcExt::with_mut`] would panic with a
  /// `BorrowMutError`.
  fn try_with_mut<F, R>(&self, f: F) -> Option<R>
  where
    F: FnOnce(&mut T) -> R,
  {
    Some(f(&mut *self.try_borrow_mut().ok()?))
  }

  /// Returns `true` if both handles share the same allocation.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let u = 1.iUrc();
  /// assert!(u.ptr_eq(&u.clone()));
  /// assert!(!u.ptr_eq(&1.iUrc()));
  /// ```
  fn ptr_eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(self, other)
  }

  /// Compare the values behind two handles.
  ///
  /// Aliased handles (`ptr_eq`) are compared through a single borrow so
  /// the call cannot trip over itself. Distinct handles behave like
  /// [`UrcExt::with`]: borrowing a contended handle panics.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::{IntoUrc, UrcExt};
  ///
  /// let a = 1.iUrc();
  /// assert!(a.val_eq(&a.clone()));
  /// assert!(!a.val_eq(&2.iUrc()));
  /// ```
  fn val_eq(&self, other: &Self) -> bool
  where
    T: PartialEq,
  {
    if Rc::ptr_eq(self, other) {
      let this = self.borrow();
      let v = &*this;
      return *v == *v;
    }

    *self.borrow() == *other.borrow()
  }
}
