#![feature(proc_macro_hygiene)]

//! `#[clone]` on a `let` binding without an initializer previously panicked
//! inside the macro; it must be a compile error instead.

use libu_derive::clone;

fn main() {
  let x = 5;
  #[clone(x)]
  let y;
}
