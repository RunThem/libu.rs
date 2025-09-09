use std::sync::Arc;

#[extend::ext(pub, name = IntoArc)]
impl<T> T {
  fn iArc(self) -> Arc<T> {
    Arc::new(self)
  }
}
