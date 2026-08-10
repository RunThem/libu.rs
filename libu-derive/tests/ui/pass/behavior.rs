//! End-to-end pass case: the generated code is actually compiled and run.
//!
//! Covers the behaviors that unit tests can only assert structurally:
//! const generics, default type params, a custom `Option` type, lazy
//! default evaluation at runtime, and the `must` panic.

use libu_derive::Builder;
use std::sync::atomic::{AtomicUsize, Ordering};

// const generic parameter
#[derive(Builder)]
struct Arr<const N: usize> {
  #[builder(must)]
  data: [u8; N],
}

// const generic parameter with a default value
#[derive(Builder)]
struct ArrD<const N: usize = 8> {
  #[builder(must)]
  data: [u8; N],
}

// default type parameter
#[derive(Builder)]
struct Def<T = i32> {
  #[builder(must)]
  val: T,
}

// custom Option type must not be mistaken for std Option
mod fake {
  pub struct Option<T>(pub T);
}

#[derive(Builder)]
struct F {
  #[builder(must)]
  f: fake::Option<u32>,
  #[builder(must)]
  s: String,
}

// lazy default evaluation, observable at runtime
static EVAL: AtomicUsize = AtomicUsize::new(0);

#[derive(Builder)]
struct Lazy {
  #[builder(default = { EVAL.fetch_add(1, Ordering::Relaxed); 42 })]
  n: u32,
}

fn main() {
  // const generics
  let a = Arr::<4>::builder().with_data([1, 2, 3, 4]).build();
  assert_eq!(a.data, [1, 2, 3, 4]);

  // const generic with a default parameter value
  let ad = ArrD::builder().with_data([1, 2]).build();
  assert_eq!(ad.data, [1, 2]);

  // default type parameter, both the default and an explicit substitution
  let d = Def::builder().with_val(3).build();
  assert_eq!(d.val, 3);
  let s = Def::<String>::builder().with_val("x".to_string()).build();
  assert_eq!(s.val, "x");

  // custom Option keeps its own type; String still defaults to Into
  let f = F::builder().with_f(fake::Option(7)).with_s("y").build();
  assert_eq!(f.f.0, 7);
  assert_eq!(f.s, "y");

  // default expression only evaluates when the field was not set
  let _ = Lazy::builder().with_n(9).build();
  assert_eq!(EVAL.load(Ordering::Relaxed), 0);
  let l = Lazy::builder().build();
  assert_eq!(l.n, 42);
  assert_eq!(EVAL.load(Ordering::Relaxed), 1);

  // must field left unset panics at build time
  let r = std::panic::catch_unwind(|| {
    let _ = Arr::<4>::builder().build();
  });
  assert!(r.is_err(), "unset must field should panic");
}
