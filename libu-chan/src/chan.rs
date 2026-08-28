//! `Chan<S, R>` — bidirectional point-to-point channel built on two flume
//! unbounded channels.

use std::any::type_name;
use std::time::Duration;

pub use flume::{
  Receiver, RecvError, RecvTimeoutError, Selector, SendError, SendTimeoutError, Sender,
  TryRecvError, TrySendError, unbounded,
};

/// 点对点线程安全的双向消息队列
///
/// Each half can only `send` values of type `S` and `recv` values of type
/// `R`; the peer returned by [`channel`] is the exact mirror, so the two
/// ends talk to each other over independent directions. The underlying
/// flume channels are unbounded: sends never block, only receives do.
pub struct Chan<S, R>(Sender<S>, Receiver<R>);

impl<S, R> Chan<S, R> {
  /// Send an `S` message to the peer.
  ///
  /// The underlying channel is unbounded, so this never blocks; it fails
  /// only if the peer has been dropped.
  pub fn send(&self, msg: S) -> Result<(), SendError<S>> {
    self.0.send(msg)
  }

  /// Send an `S` message without blocking.
  ///
  /// On an unbounded channel this behaves like [`Chan::send`] except for
  /// the error type (never `Full`); it exists so a future bounded `Chan`
  /// can keep the same API surface.
  pub fn try_send(&self, msg: S) -> Result<(), TrySendError<S>> {
    self.0.try_send(msg)
  }

  /// Send an `S` message, giving up after `timeout`.
  ///
  /// On an unbounded channel the send itself cannot stall, so the only
  /// observable failure is a disconnected peer.
  pub fn send_timeout(&self, msg: S, timeout: Duration) -> Result<(), SendTimeoutError<S>> {
    self.0.send_timeout(msg, timeout)
  }

  /// Receive an `R` message from the peer, blocking until one arrives.
  ///
  /// Returns `Err(RecvError::Disconnected)` once the peer is dropped and
  /// the channel is drained.
  pub fn recv(&self) -> Result<R, RecvError> {
    self.1.recv()
  }

  /// Receive an `R` message without blocking.
  ///
  /// Returns `Err(TryRecvError::Empty)` when no message is queued yet.
  pub fn try_recv(&self) -> Result<R, TryRecvError> {
    self.1.try_recv()
  }

  /// Receive an `R` message, giving up after `timeout`.
  ///
  /// Returns `Err(RecvTimeoutError::Timeout)` when nothing arrives in
  /// time.
  pub fn recv_timeout(&self, timeout: Duration) -> Result<R, RecvTimeoutError> {
    self.1.recv_timeout(timeout)
  }

  /// Receiving iterator that yields until the peer disconnects.
  pub fn iter(&self) -> impl Iterator<Item = R> + '_ {
    std::iter::from_fn(|| self.recv().ok())
  }

  /// Split the handle back into the raw flume halves.
  ///
  /// Useful to hand the endpoints to APIs [`Chan`] does not wrap, e.g.
  /// flume's [`Selector`] for multiplexing several channels.
  ///
  /// # Example
  ///
  /// ```rust
  /// use libu_chan::channel;
  ///
  /// let (a, b) = channel::<u8, u8>();
  /// let (tx, rx) = a.disassemble();
  ///
  /// tx.send(1).unwrap();
  /// assert_eq!(b.recv(), Ok(1));
  /// ```
  pub fn disassemble(self) -> (Sender<S>, Receiver<R>) {
    (self.0, self.1)
  }

  /// Escape hatch to the raw sending half.
  pub fn tx(&self) -> &Sender<S> {
    &self.0
  }

  /// Escape hatch to the raw receiving half.
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
    write!(f, "{}", type_name::<Self>())
  }
}

/// Create a connected pair of bidirectional channels.
///
/// Whatever `a` sends, `b` receives, and vice versa:
///
/// ```rust
/// use libu_chan::channel;
///
/// let (a, b) = channel::<&str, u8>();
///
/// a.send("ping").unwrap();
/// assert_eq!(b.recv(), Ok("ping"));
///
/// b.send(42).unwrap();
/// assert_eq!(a.recv(), Ok(42));
/// ```
pub fn channel<S, R>() -> (Chan<S, R>, Chan<R, S>) {
  let (t0, r0) = unbounded::<S>();
  let (t1, r1) = unbounded::<R>();

  (Chan(t0, r1), Chan(t1, r0))
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::*;

  #[test]
  fn roundtrip_both_directions() {
    let (a, b) = channel::<u8, i32>();

    a.send(1).unwrap();
    assert_eq!(b.recv(), Ok(1));

    b.send(-2).unwrap();
    assert_eq!(a.recv(), Ok(-2));
  }

  #[test]
  fn try_send_reports_disconnect() {
    let (a, b) = channel::<u8, u8>();
    drop(b);
    assert!(matches!(a.try_send(1), Err(TrySendError::Disconnected(_))));
  }

  #[test]
  fn timeouts() {
    let (a, b) = channel::<u8, u8>();

    assert!(matches!(
      b.recv_timeout(Duration::from_millis(50)),
      Err(RecvTimeoutError::Timeout)
    ));

    a.send(7).unwrap();
    assert_eq!(b.recv_timeout(Duration::from_millis(50)), Ok(7));

    drop(b);
    assert!(matches!(
      a.send_timeout(1, Duration::from_millis(50)),
      Err(SendTimeoutError::Disconnected(_))
    ));
  }

  #[test]
  fn disassemble_keeps_the_link() {
    let (a, b) = channel::<u8, u8>();
    let (tx, rx) = a.disassemble();

    b.send(3).unwrap();
    assert_eq!(rx.recv(), Ok(3));

    tx.send(4).unwrap();
    assert_eq!(b.recv(), Ok(4));
  }

  #[test]
  fn iter_drains_until_disconnect() {
    let (a, b) = channel::<u8, u8>();

    a.send(1).unwrap();
    a.send(2).unwrap();
    drop(a);

    assert_eq!(b.iter().collect::<Vec<_>>(), vec![1, 2]);
  }

  #[test]
  fn debug_shows_type_name() {
    let (a, _) = channel::<u8, u8>();
    assert_eq!(format!("{a:?}"), "libu_chan::chan::Chan<u8, u8>");
  }
}
