use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub type Urc<T> = Rc<RefCell<T>>;

#[extend::ext(pub, name = IntoUrc)]
impl<T> T {
  fn iUrc(self) -> Urc<T> {
    Rc::new(RefCell::new(self))
  }
}

#[extend::ext(pub, name = AtUrc)]
impl<T> Urc<T> {
  fn at(&self) -> Ref<'_, T> {
    self.borrow()
  }

  fn at_mut(&self) -> RefMut<'_, T> {
    self.borrow_mut()
  }
}
