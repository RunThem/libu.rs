use flume::{Receiver, RecvError, SendError, Sender, TryRecvError, unbounded};

/// 点对点线程安全的双向消息队列
pub struct Chan<S, R>(Sender<S>, Receiver<R>);

impl<S, R> Chan<S, R> {
  pub fn send(&self, msg: S) -> Result<(), SendError<S>> {
    self.0.send(msg)
  }

  pub fn recv(&self) -> Result<R, RecvError> {
    self.1.recv()
  }

  pub fn try_recv(&self) -> Result<R, TryRecvError> {
    self.1.try_recv()
  }

  pub fn iter(&self) -> impl Iterator<Item = R> + '_ {
    std::iter::from_fn(|| self.recv().ok())
  }

  pub fn tx(&self) -> &Sender<S> {
    &self.0
  }

  pub fn rx(&self) -> &Receiver<R> {
    &self.1
  }
}

impl<S, R> AsRef<Sender<S>> for Chan<S, R> {
  fn as_ref(&self) -> &Sender<S> {
    &self.0
  }
}

impl<S, R> AsRef<Receiver<R>> for Chan<S, R> {
  fn as_ref(&self) -> &Receiver<R> {
    &self.1
  }
}

impl<S, R> std::fmt::Debug for Chan<S, R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", tynm::type_name::<Chan<S, R>>())
  }
}

pub fn channel<S, R>() -> (Chan<S, R>, Chan<R, S>) {
  let (t0, r0) = unbounded::<S>();
  let (t1, r1) = unbounded::<R>();

  (Chan(t0, r1), Chan(t1, r0))
}
