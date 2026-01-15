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
impl<T: Clone> Urc<T> {
  fn at(&self) -> T {
    self.borrow().clone()
  }
}

#[extend::ext(pub, name = WithUrc)]
impl<T> Urc<T> {
  fn with<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&T) -> R,
  {
    f(&*self.borrow())
  }

  fn with_mut<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&mut T) -> R,
  {
    f(&mut *self.borrow_mut())
  }
}
