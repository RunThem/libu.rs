use std::sync::{Arc, Mutex, MutexGuard};

pub type Mrc<T> = Arc<Mutex<T>>;

#[extend::ext(pub, name = IntoMrc)]
impl<T> T {
  fn iMrc(self) -> Mrc<T> {
    Arc::new(Mutex::new(self))
  }
}

#[extend::ext(pub, name = AtMrc)]
impl<T> Mrc<T> {
  fn at(&self) -> MutexGuard<'_, T> {
    self.lock().unwrap()
  }
}
