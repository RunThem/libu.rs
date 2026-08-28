//! Reset-to-default for `Default` types.

use extend::ext;

/// Reset to default value
///
/// Resets the value to its type's default, equivalent to `*self = Default::default()`.
///
/// # Example
///
/// ```rust
/// use libu_trait::Bzero;
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
impl<T: Default> T {
  #[inline]
  fn bzero(&mut self) {
    *self = Default::default()
  }
}
