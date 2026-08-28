//! Trait extensions module
//!
//! Provides a set of trait extension methods implemented via the `extend`
//! crate, adding convenient operations for standard types. One module per
//! extension; call sites usually reach them through the umbrella prelude
//! (`libu::prelude::*`) or by importing the concrete trait.
//!
//! # Available Extensions
//!
//! | Type | Method | Description |
//! |------|------|------|
//! | `bool` | [`Pick::pick`] | Ternary selector |
//! | `T: Default` | [`Bzero::bzero`] | Reset to default value |
//! | `T: Sized` | [`Void::void`] | Suppress must_use warnings |
//! | `str` | [`ToDur::to_dur`] / [`ToDur::try_to_dur`] | Parse string to `Duration` |
//! | `u64` | [`DurExt`] | Duration literals (`5.secs()`, `250.ms()`, …) |
//! | `Vec<T>` | [`RemoveIf::remove_if`] | Remove elements by condition |
//! | `T: Debug` | [`Pretty::pretty`] | Pretty-print with 2-space indent |

mod bzero;
mod dur;
mod pick;
mod pretty;
mod remove_if;
mod void;

pub use bzero::*;
pub use dur::*;
pub use pick::*;
pub use pretty::*;
pub use remove_if::*;
pub use void::*;
