use libu_derive::Builder;

#[derive(Builder)]
struct S {
  #[builder(must, default = 1)]
  x: u32,
}

fn main() {}
