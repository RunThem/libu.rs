use libu_derive::Builder;

#[derive(Builder)]
struct S {
  #[builder(bogus)]
  x: u32,
}

fn main() {}
