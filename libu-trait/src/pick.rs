//! Ternary selector for `bool`.

use extend::ext;

/// Ternary selector
///
/// Returns one of two values based on a boolean condition. For an
/// `Option`-producing variant see `bool::then_some`.
///
/// # Example
///
/// ```rust
/// use libu_trait::Pick;
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
