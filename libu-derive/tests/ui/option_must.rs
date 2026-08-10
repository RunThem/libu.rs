use libu_derive::Builder;

#[derive(Builder)]
struct S {
  #[builder(must)]
  x: Option<u32>,
}

fn main() {}
