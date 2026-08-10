use libu_derive::Builder;

#[derive(Builder)]
struct S {
  #[builder(must, skip)]
  x: u32,
}

fn main() {}
