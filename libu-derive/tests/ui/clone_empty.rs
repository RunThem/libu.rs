#![feature(proc_macro_hygiene)]

//! `#[clone]` / `#[clone()]` with no identifiers would clone nothing. It must
//! require at least one identifier.

use libu_derive::clone;

fn main() {
  let a = 5;
  #[clone()]
  let b = a;
}
