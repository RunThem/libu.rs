use std::rc::Rc;

#[extend::ext(pub, name = IntoRc)]
impl<T> T {
  fn iRc(self) -> Rc<T> {
    Rc::new(self)
  }
}
