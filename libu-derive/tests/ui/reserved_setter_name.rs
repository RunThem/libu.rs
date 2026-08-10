use libu_derive::Builder;

#[derive(Builder)]
struct S {
  #[builder(prefix = "")]
  build: u32,
}

fn main() {}
