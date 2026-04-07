//! Trait extensions module
//!
//! Provides a set of trait extension methods implemented via the `extend` crate,
//! adding convenient operations for standard types.
//!
//! # Available Extensions
//!
//! | Type | Method | Description |
//! |------|------|------|
//! | `bool` | [`pick`] | Ternary selector |
//! | `T: Default` | [`bzero`] | Reset to default value |
//! | `T: Sized` | [`void`] | Suppress must_use warnings |
//! | `str` | [`to_dur`] | Parse string to Duration |
//! | `Vec<T>` | [`remove_if`] | Remove elements by condition |

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use extend::ext;

/// Ternary selector
///
/// Returns one of two values based on a boolean condition.
///
/// # Example
///
/// ```rust
/// use libu::Pick;
///
/// let value = true.pick(1, 2);
/// assert_eq!(value, 1);
///
/// let value = false.pick("yes", "no");
/// assert_eq!(value, "no");
/// ```
#[ext(pub, name = Pick)]
impl<O> bool {
  #[inline]
  fn pick(self, if_true: O, if_false: O) -> O {
    if self { if_true } else { if_false }
  }
}

/// Reset to default value
///
/// Resets the value to its type's default, equivalent to `*self = Default::default()`.
///
/// # Example
///
/// ```rust
/// use libu::Bzero;
///
/// let mut value = 42;
/// value.bzero();
/// assert_eq!(value, 0);
///
/// let mut s = String::from("hello");
/// s.bzero();
/// assert_eq!(s, "");
/// ```
#[ext(pub, name = Bzero)]
impl<A: Default> A {
  #[inline]
  fn bzero(&mut self) {
    *self = Default::default()
  }
}

/// Consume and discard value
///
/// Explicitly consumes a value to suppress `must_use` warnings.
/// Useful when you need to ignore a return value without compiler warnings.
///
/// # Example
///
/// ```rust
/// use libu::Void;
///
/// // Some function returns Result, but we don't care about the result
/// fn some_fn() -> Result<(), ()> { Ok(()) }
///
/// // Direct call would produce a must_use warning
/// // some_fn(); // warning: unused `Result`
///
/// // Use void to explicitly consume
/// some_fn().void();
/// ```
#[ext(pub, name = Void)]
impl<B: Sized> B {
  #[inline]
  fn void(self) {}
}

/// Parse string to Duration
///
/// Parses a time string into a `Duration`.
///
/// # Supported Units
///
/// | Unit | Description |
/// |------|------|
/// | `ns` | Nanoseconds |
/// | `us` | Microseconds |
/// | `ms` | Milliseconds |
/// | `s` | Seconds |
/// | `m` | Minutes |
///
/// # Panics
///
/// - Panics if the numeric part cannot be parsed as `u64`
/// - Panics if the unit is not in the supported list
///
/// # Example
///
/// ```rust
/// use libu::ToDur;
/// use std::time::Duration;
///
/// let dur = "100ms".to_dur();
/// assert_eq!(dur, Duration::from_millis(100));
///
/// let dur = "5s".to_dur();
/// assert_eq!(dur, Duration::from_secs(5));
///
/// let dur = "2m".to_dur();
/// assert_eq!(dur, Duration::from_secs(120));
/// ```
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

/// Remove elements by condition
///
/// Removes elements that satisfy the predicate and returns them as a new Vec.
///
/// # Complexity
///
/// Time complexity is **O(n²)** because each `Vec::remove` operation is O(n).
/// For large collections, consider using `retain` or `drain` instead.
///
/// # Example
///
/// ```rust
/// use libu::RemoveIf;
///
/// let mut vec = vec![1, 2, 3, 4, 5, 6];
/// let removed = vec.remove_if(|x| x % 2 == 0);
///
/// assert_eq!(vec, vec![1, 3, 5]);
/// assert_eq!(removed, vec![2, 4, 6]);
/// ```
#[ext(pub, name = RemoveIf)]
impl<T, F: Fn(&T) -> bool> Vec<T> {
  fn remove_if(&mut self, predicate: F) -> Self {
    let mut i = 0;
    let mut removed = Self::with_capacity(self.len());

    while i < self.len() {
      if predicate(&self[i]) {
        removed.push(self.remove(i));
      } else {
        i += 1;
      }
    }

    removed
  }
}