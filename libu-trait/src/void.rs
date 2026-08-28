//! Consume-and-discard for `must_use` values.

use extend::ext;

/// Consume and discard value
///
/// Explicitly consumes a value to suppress `must_use` warnings.
/// Useful when you need to ignore a return value without compiler
/// warnings. Equivalent to `std::mem::drop`, but reads naturally in
/// method chains.
///
/// # Example
///
/// ```rust
/// use libu_trait::Void;
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
impl<T: Sized> T {
  #[inline]
  fn void(self) {}
}
