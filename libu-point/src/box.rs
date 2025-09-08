#[extend::ext(pub, name=IntoBox)]
impl<T> T {
  fn iBox(self) -> Box<T> {
    Box::new(self)
  }
}
