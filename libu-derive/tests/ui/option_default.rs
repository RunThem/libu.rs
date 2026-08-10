use libu_derive::Builder;

#[derive(Builder)]
struct S {
  #[builder(default = Some(5))]
  x: Option<u32>,
}

fn main() {}
