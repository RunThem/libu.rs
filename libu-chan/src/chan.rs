use std::sync::mpsc;

/// 点对点线程安全的双向消息队列
pub struct Chan<X, Y>(mpsc::Sender<X>, mpsc::Receiver<Y>);

impl<X, Y> Chan<X, Y> {
  pub fn send(&self, msg: X) -> Result<(), mpsc::SendError<X>> {
    self.0.send(msg)
  }

  pub fn recv(&self) -> Result<Y, mpsc::RecvError> {
    self.1.recv()
  }

  pub fn try_recv(&self) -> Result<Y, mpsc::TryRecvError> {
    self.1.try_recv()
  }
}

pub fn channel<X, Y>() -> (Chan<X, Y>, Chan<Y, X>) {
  let (t0, r0) = mpsc::channel::<X>();
  let (t1, r1) = mpsc::channel::<Y>();

  (Chan(t0, r1), Chan(t1, r0))
}
