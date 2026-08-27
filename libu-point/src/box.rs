//! `Box` construction sugar.
//!
//! | Trait | Method | Description |
//! |-------|--------|-------------|
//! | [`IntoBox`] | [`iBox`](IntoBox::iBox) | Wrap a value in a `Box` |

#[extend::ext(pub, name=IntoBox)]
impl<T> T {
  /// Wrap a value in a heap-allocated [`Box`](std::boxed::Box).
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_point::IntoBox;
  ///
  /// let b = 1.iBox();
  /// assert_eq!(*b, 1);
  /// ```
  fn iBox(self) -> Box<T> {
    Box::new(self)
  }
}
