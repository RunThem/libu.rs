#![allow(unused)]
#![allow(non_snake_case)]

mod builder;

use crate::builder::BuilderDeriveInput;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  BuilderDeriveInput::from_derive_input(&input)
    .unwrap()
    .to_token_stream()
    .into()
}

#[proc_macro_derive(Sync)]
pub fn derive_sync(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  let ident = input.ident;

  quote::quote! (unsafe impl Sync for #ident {}).into()
}

#[proc_macro_derive(Send)]
pub fn derive_send(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  let ident = input.ident;

  quote::quote! (unsafe impl Send for #ident {}).into()
}
