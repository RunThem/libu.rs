//! Procedural macros module
//!
//! Provides derive macros and attribute macros for common patterns.
//!
//! # Available Macros
//!
//! ## Derive Macros
//!
//! | Macro | Description |
//! |-------|-------------|
//! | [`Builder`] | Generate builder pattern for structs |
//! | [`Send`] | **unsafe** - Implement `Send` trait |
//! | [`Sync`] | **unsafe** - Implement `Sync` trait |
//!
//! ## Attribute Macros
//!
//! | Macro | Description |
//! |-------|-------------|
//! | [`clone`] | Auto-clone variables in closures |
//!
//! # Safety Warning
//!
//! The `Send` and `Sync` derive macros use `unsafe impl` to forcefully implement
//! these traits for your types. This bypasses Rust's safety guarantees and can
//! lead to undefined behavior if your type is not actually thread-safe.
//!
//! **Use these macros only when you are certain your type is safe to share
//! across threads.**

#![allow(unused)]
#![allow(non_snake_case)]

mod builder;
mod clone;
mod select;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::ToTokens;

/// Generate builder pattern for structs
///
/// Creates a `TypeNameBuilder` struct with setter methods for each field,
/// and adds a `builder()` method to the original struct.
///
/// # Field Attributes
///
/// - `#[builder(into)]` - Accept `impl Into<T>` in setter method
/// - `#[builder(must)]` - Field must be initialized (panics if not set)
///
/// # Behavior
///
/// - `Option<T>` fields: kept as Option, no default required
/// - Other fields: use `Default::default()` if not set, unless `#[builder(must)]`
///
/// # Example
///
/// ```rust
/// use libu::Builder;
///
/// #[derive(Builder)]
/// struct Config {
///   name: String,
///   #[builder(into)]
///   path: String,
///   timeout: Option<u64>,
///   #[builder(must)]
///   required_field: i32,
/// }
///
/// let config = Config::builder()
///   .name("app")
///   .path("/tmp/config")
///   .required_field(42)
///   .build();
/// ```
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  builder::BuilderDeriveInput::from_derive_input(&input)
    .unwrap()
    .to_token_stream()
    .into()
}

/// **unsafe** - Implement `Sync` trait
///
/// Forcefully implements `Sync` for a type using `unsafe impl`.
///
/// # Safety
///
/// This macro bypasses Rust's automatic `Sync` verification. You must ensure
/// that your type is actually safe to share across threads. Improper use can
/// cause data races and undefined behavior.
///
/// # Example
///
/// ```rust
/// use libu::Sync;
///
/// // WARNING: Only use if you know this is safe!
/// #[derive(Sync)]
/// struct MyType {
///   // fields must be thread-safe
/// }
/// ```
#[proc_macro_derive(Sync)]
pub fn derive_sync(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  let ident = input.ident;

  quote::quote! {
    /// SAFETY: The user has explicitly opted into this unsafe implementation.
    /// They must ensure this type is actually safe to share across threads.
    unsafe impl Sync for #ident {}
  }
  .into()
}

/// **unsafe** - Implement `Send` trait
///
/// Forcefully implements `Send` for a type using `unsafe impl`.
///
/// # Safety
///
/// This macro bypasses Rust's automatic `Send` verification. You must ensure
/// that your type is actually safe to transfer across threads. Improper use can
/// cause data races and undefined behavior.
///
/// # Example
///
/// ```rust
/// use libu::Send;
///
/// // WARNING: Only use if you know this is safe!
/// #[derive(Send)]
/// struct MyType {
///   // fields must be safe to move across threads
/// }
/// ```
#[proc_macro_derive(Send)]
pub fn derive_send(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);
  let ident = input.ident;

  quote::quote! {
    /// SAFETY: The user has explicitly opted into this unsafe implementation.
    /// They must ensure this type is actually safe to transfer across threads.
    unsafe impl Send for #ident {}
  }
  .into()
}

/// Auto-clone variables in closures
///
/// Automatically clones specified variables before using them in a closure
/// or expression. Useful for capturing variables by clone instead of reference.
///
/// # Note
///
/// Requires `feature(proc_macro_hygiene)` in your crate.
///
/// # Example
///
/// ```rust
/// use libu::clone;
///
/// let data = vec![1, 2, 3];
/// let name = String::from("test");
///
/// // Clone `data` and `name` before the closure
/// #[clone(data, name)]
/// let handle = thread::spawn(|| {
///   println!("data: {:?}", data);
///   println!("name: {}", name);
/// });
///
/// // Works with expressions too
/// #[clone(data)]
/// let result = { data.len() };
/// ```
#[proc_macro_attribute]
pub fn clone(attr: TokenStream, item: TokenStream) -> TokenStream {
  clone::clone(attr, item)
}

/// Wait on multiple channel operations simultaneously.
///
/// Expands each arm into a `flume::Selector::new().recv(...).recv(...).wait()` chain,
/// allowing a thread to block until one of the registered channels becomes ready.
///
/// # Syntax
///
/// ```rust,ignore
/// select! [
///   &rx1 => |msg| { /* handle rx1 */ },
///   &rx2 => |msg| { /* handle rx2 */ },
/// ];
/// ```
///
/// # Expansion
///
/// ```rust,ignore
/// // Expands to:
/// ::flume::Selector::new()
///   .recv(&rx1, |msg| { /* handle rx1 */ })
///   .recv(&rx2, |msg| { /* handle rx2 */ })
///   .wait();
/// ```
#[proc_macro]
pub fn select(item: TokenStream) -> TokenStream {
  select::select(item)
}
