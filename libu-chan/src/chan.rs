use std::sync::mpsc;

/// 点对点线程安全的双向消息队列
pub struct Chan<S, R>(mpsc::Sender<S>, mpsc::Receiver<R>);

impl<S, R> Chan<S, R> {
  pub fn send(&self, msg: S) -> Result<(), mpsc::SendError<S>> {
    self.0.send(msg)
  }

  pub fn recv(&self) -> Result<R, mpsc::RecvError> {
    self.1.recv()
  }

  pub fn try_recv(&self) -> Result<R, mpsc::TryRecvError> {
    self.1.try_recv()
  }

  pub fn iter(&self) -> impl Iterator<Item = R> + '_ {
    std::iter::from_fn(|| self.recv().ok())
  }
}

impl<S, R> std::fmt::Debug for Chan<S, R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", tynm::type_name::<Chan<S, R>>())
  }
}

impl<S, R> std::fmt::Display for Chan<S, R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self)
  }
}

pub fn channel<S, R>() -> (Chan<S, R>, Chan<R, S>) {
  let (t0, r0) = mpsc::channel::<S>();
  let (t1, r1) = mpsc::channel::<R>();

  (Chan(t0, r1), Chan(t1, r0))
}
