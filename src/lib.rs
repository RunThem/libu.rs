#![allow(non_snake_case)]
#![allow(unused)]

pub use libu_chan::*;
pub use libu_derive::*;
pub use libu_log::*;
pub use libu_macro::*;
pub use libu_point::*;
pub use libu_timer::*;
pub use libu_trait::*;

mod test {
  use super::*;

  #[test]
  fn tset() {
    let __ = 0.iBox();
    let __ = 0.iUrc();
    let __ = 0.iMrc();
  }
}
