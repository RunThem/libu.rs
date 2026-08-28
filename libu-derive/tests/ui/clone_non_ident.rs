#![feature(proc_macro_hygiene)]

//! `#[clone(a.b)]` was silently split into two clones of `a` and `b`
//! (non-identifier tokens were dropped). Non-identifier tokens must be
//! rejected.

use libu_derive::clone;

fn main() {
  let a = 1;
  #[clone(a.b)]
  let c = a;
}
