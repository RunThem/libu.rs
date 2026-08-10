use libu_derive::Builder;

#[derive(Builder)]
struct S {
  x: Option<u32>,
  // with_x_opt collides with the dual setter generated for `x`
  x_opt: u32,
}

fn main() {}
