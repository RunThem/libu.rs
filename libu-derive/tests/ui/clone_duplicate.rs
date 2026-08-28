#![feature(proc_macro_hygiene)]

//! `#[clone(a, a)]` previously generated a redundant double clone. Duplicate
//! identifiers must be rejected.

use libu_derive::clone;

fn main() {
  let a = 5;
  #[clone(a, a)]
  let b = a;
}
