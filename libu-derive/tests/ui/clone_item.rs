#![feature(proc_macro_hygiene)]

//! `#[clone]` on an item (here a `fn`) was a silent no-op. It must be
//! rejected.

use libu_derive::clone;

#[clone(x)]
fn foo() {}

fn main() {}
