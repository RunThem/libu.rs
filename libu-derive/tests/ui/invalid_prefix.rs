use libu_derive::Builder;

#[derive(Builder)]
struct S {
  #[builder(prefix = "with-")]
  x: u32,
}

fn main() {}
