//! Unsafe `Sync` derive codegen — shared implementation in [`crate::marker`].

use proc_macro2::TokenStream;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
  super::marker::expand(input, "Sync")
}
