#![allow(non_snake_case)]

pub mod prelude {
  pub use libu_chan::*;
  pub use libu_derive::*;
  pub use libu_macro::*;
  pub use libu_point::*;
  pub use libu_trait::*;
}

pub mod dependency {
  pub mod parking_lot {
    pub use parking_lot::*;
  }

  pub mod flume {
    pub use flume::*;
  }
}

#[cfg(test)]
mod test {
  use super::prelude::*;

  /// Smoke-test the umbrella prelude: pull one representative item through
  /// `libu::prelude::*` from every member crate, so a broken re-export or a
  /// glob collision shows up here instead of at call sites.
  #[test]
  fn prelude_reexports() {
    // libu-point: the iXxx constructors
    let _ = 0.iBox();
    let _ = 0.iUrc();
    let _ = 0.iMrc();

    // libu-trait: duration parsing, numeric literals, and the ternary selector
    let _ = "1d".try_to_dur().unwrap();
    let _ = 5.secs();
    let _ = true.pick(1, 2);

    // libu-macro: map and hash macros
    let _ = hmap! { "a" => 1 };
    let _ = hash!(5u64);

    // libu-chan: build a channel
    let (_a, _b) = channel::<u8, u8>();
  }
}
