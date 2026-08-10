use libu_derive::Builder;

#[derive(Builder)]
struct S {
  // prefix + field name spell the reserved keyword `abstract`
  #[builder(prefix = "abstrac")]
  t: u32,
}

fn main() {}
