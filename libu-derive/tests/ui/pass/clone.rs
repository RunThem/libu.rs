#![feature(proc_macro_hygiene)]
#![feature(stmt_expr_attributes)]

//! End-to-end pass case: `#[clone]` inserts `clone()` calls before a `let`
//! statement or an expression, so a copy moves into a closure / another
//! thread while the original keeps its own value.

use libu_derive::clone;
use std::thread;

fn main() {
  // let-statement form with several identifiers: clone before the closure.
  let data = vec![1, 2, 3];
  let name = String::from("test");

  #[clone(data, name)]
  let handle = thread::spawn(move || {
    assert_eq!(data, [1, 2, 3]);
    assert_eq!(name, "test");
  });
  handle.join().unwrap();

  // The originals are untouched — the macro cloned, it did not move.
  assert_eq!(data, [1, 2, 3]);
  assert_eq!(name, "test");

  // Expression-statement form: the block consumes the shadowing clone, so
  // the original `data` keeps its contents.
  #[clone(data)]
  {
    assert_eq!(data.into_iter().count(), 3);
  }
  assert_eq!(data, [1, 2, 3]);
}
