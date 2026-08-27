#![allow(non_snake_case)]
#![allow(unused)]
#![feature(proc_macro_hygiene)]

pub use libu_chan::*;
pub use libu_derive::*;
pub use libu_macro::*;
pub use libu_point::*;
pub use libu_timer::*;
pub use libu_trait::*;

pub mod dependency {
  pub mod parking_lot {
    pub use parking_lot::*;
  }

  pub mod flume {
    pub use flume::*;
  }
}

mod test {
  use super::*;

  #[test]
  fn tset() {
    let __ = 0.iBox();
    let __ = 0.iUrc();
    let __ = 0.iMrc();
  }
}
