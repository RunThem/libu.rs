use std::sync::{Arc, Mutex, MutexGuard};

pub type Mrc<T> = Arc<Mutex<T>>;

#[extend::ext(pub, name = IntoMrc)]
impl<T> T {
  fn iMrc(self) -> Mrc<T> {
    Arc::new(Mutex::new(self))
  }
}

#[extend::ext(pub, name = AtMrc)]
impl<T: Clone> Mrc<T> {
  fn at(&self) -> T {
    self
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone()
  }
}

#[extend::ext(pub, name = WithMrc)]
impl<T> Mrc<T> {
  fn with<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&T) -> R,
  {
    f(&*self.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
  }

  fn with_mut<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&mut T) -> R,
  {
    f(&mut *self.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
  }
}
