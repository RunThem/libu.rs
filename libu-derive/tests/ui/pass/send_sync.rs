//! End-to-end pass case: `#[derive(Send, Sync)]` force-implements the traits
//! even though the field types are neither — the generated `unsafe impl`s must
//! compile (including with generics) and the type must cross thread boundaries.

use libu_derive::{Send, Sync};
use std::rc::Rc;

#[derive(Send, Sync)]
struct Shared {
  // Rc<()> is neither Send nor Sync; only the derives make this compile.
  marker: Rc<()>,
}

// Generic types: the impl must carry the type parameters through.
#[derive(Send, Sync)]
struct Generic<T>(T);

// Lifetimes: the impl must carry the lifetime parameter through.
#[derive(Send, Sync)]
struct Borrowed<'a>(&'a Rc<()>);

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
  // Compile-time proof that the traits were implemented.
  assert_send::<Shared>();
  assert_sync::<Shared>();

  // Generic and lifetime-carrying types work too.
  assert_send::<Generic<Rc<()>>>();
  assert_sync::<Generic<Rc<()>>>();
  assert_send::<Borrowed<'_>>();
  assert_sync::<Borrowed<'_>>();
  let rc = Rc::new(());
  let borrowed = Borrowed(&rc);
  let _ = borrowed.0;

  // Send in action: the value moves to another thread (requires Send).
  let shared = Shared { marker: Rc::new(()) };
  let handle = std::thread::spawn(move || {
    let _ = shared.marker;
  });
  handle.join().unwrap();

  // ...and so does a generic instance.
  let generic = Generic(Rc::new(()));
  let handle = std::thread::spawn(move || {
    let _ = generic.0;
  });
  handle.join().unwrap();
}
