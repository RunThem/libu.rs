use libu_derive::Builder;

#[derive(Builder)]
struct St {
  name: String,
}

fn main() {
  // Builder fields are encapsulated: only with_name() can touch them.
  St::builder().name = None;
}
