pub trait IntoBox<T> {
  fn iBox(self) -> Box<T>;
}

impl<T> IntoBox<T> for T {
  fn iBox(self) -> Box<T> {
    Box::new(self)
  }
}
