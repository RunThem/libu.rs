use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub type Urc<T> = Rc<RefCell<T>>;

pub trait IntoUrc<T> {
  fn iUrc(self) -> Urc<T>;
}

impl<T> IntoUrc<T> for T {
  fn iUrc(self) -> Urc<T> {
    Rc::new(RefCell::new(self))
  }
}

pub trait AtUrc<T> {
  fn at(&self) -> Ref<'_, T>;

  fn at_mut(&self) -> RefMut<'_, T>;

  // fn at_with<F: FnOnce(RefMut<'_, T>)>(&self, f: F);
}

impl<T> AtUrc<T> for Urc<T> {
  fn at(&self) -> Ref<'_, T> {
    self.borrow()
  }

  fn at_mut(&self) -> RefMut<'_, T> {
    self.borrow_mut()
  }

  // fn at_with<F: FnOnce(RefMut<'_, T>)>(&self, f: F) {
  //   f(self.borrow_mut())
  // }
}
