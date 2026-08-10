//! Procedural macros module
//!
//! Provides derive macros, attribute macros, and function-like macros for common patterns.
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
//! ## Function-like Macros
//!
//! | Macro | Description |
//! |-------|-------------|
//! | [`select!`] | Wait on multiple channel operations simultaneously |
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
mod send;
mod sync;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::ToTokens;

/// Generate builder pattern for structs
///
/// Creates a `TypeNameBuilder` struct with setter methods for each field,
/// and adds a `builder()` method to the original struct.
///
/// # Setter Naming
///
/// Setters are named `with_<field>` by default. `#[builder(prefix = "...")]`
/// overrides the prefix per field; an empty string disables it:
///
/// - `#[builder(prefix = "set_")]` - generate `set_<field>` instead of `with_<field>`
/// - `#[builder(prefix = "")]` - generate `<field>` with no prefix
///
/// The prefix must produce a valid identifier (`prefix = "with-"` is a
/// compile error), and a setter that would collide with the generated
/// `build()`/`default()` methods is rejected.
///
/// # Field Attributes
///
/// - `#[builder(into)]` - Accept `impl Into<T>` in setter method.
///   Defaults to on for `String` fields, so both `&str` and `String` work;
///   `#[builder(into = false)]` opts a `String` field back out of it
/// - `#[builder(must)]` - Field must be initialized (panics if not set).
///   Mutually exclusive with `skip` and `default = ...` (compile error)
/// - `#[builder(default = <expr>)]` - Use `<expr>` as the default when the field
///   is not set, e.g. for enums without `Default`: `#[builder(default = Mode::Test)]`.
///   The expression is evaluated lazily, only when the field was not set.
///   Not allowed on `Option<T>` fields (compile error); mutually exclusive with `must`
/// - `#[builder(skip)]` - Keep the field in the builder but generate no setter,
///   useful with `#[builder(default = ...)]` for internal state.
///   Mutually exclusive with `must`
/// - `#[builder(private)]` - Setter is `pub(crate)` instead of `pub`
///
/// # Behavior
///
/// - `Option<T>` fields: kept as Option, no default required; get two setters,
///   `with_<field>(T)` and `with_<field>_opt(Option<T>)`. `must`/`default` are
///   rejected on them (compile error) — pass values via the setters instead
/// - Other fields: use `Default::default()` if not set, unless `#[builder(must)]`
///   or `#[builder(default = ...)]` overrides it; a missing `Default` impl is
///   reported with a diagnostic suggesting `must` or `default = ...`
/// - Generics: lifetimes, type parameters (with bounds or defaults), const
///   generics, and where clauses are supported on both the struct and the builder
///
/// # Example
///
/// ```rust
/// use libu_derive::Builder;
///
/// enum Mode {
///   Test,
///   Release,
/// }
///
/// #[derive(Builder)]
/// struct Config {
///   name: String,
///   #[builder(must)]
///   path: String,
///   timeout: Option<u64>,
///   #[builder(default = Mode::Test)]
///   mode: Mode,
/// }
///
/// let config = Config::builder()
///   .with_name("app")
///   .with_path("/tmp/config")
///   .with_timeout(100)
///   .build();
/// ```
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  match builder::BuilderDeriveInput::from_derive_input(&input) {
    Ok(b) => b.to_token_stream().into(),
    Err(e) => e.write_errors().into(),
  }
}

/// **unsafe** - Implement `Sync` trait
///
/// Forcefully implements `Sync` for a type using `unsafe impl`.
///
/// Generics are supported: type and lifetime parameters and where clauses
/// are carried through to the generated impl.
///
/// # Safety
///
/// This macro bypasses Rust's automatic `Sync` verification. You must ensure
/// that your type is actually safe to share across threads. Improper use can
/// cause data races and undefined behavior.
///
/// Note: if every field is already `Sync`, the type is automatically `Sync`
/// and this derive is redundant.
///
/// Note: the generated impl is exempt from `#![forbid(unsafe_code)]`, like
/// all derive-generated code — the guard does not flag it. Audit the types
/// you apply this derive to yourself.
///
/// # Example
///
/// ```rust
/// use libu::Sync;
/// use std::rc::Rc;
///
/// // WARNING: Only use if you know this is safe!
/// // `Rc` is neither `Send` nor `Sync`; the derive forces it.
/// #[derive(Sync)]
/// struct MyType {
///   marker: Rc<()>,
/// }
/// ```
#[proc_macro_derive(Sync)]
pub fn derive_sync(input: TokenStream) -> TokenStream {
  sync::expand(input.into()).into()
}

/// **unsafe** - Implement `Send` trait
///
/// Forcefully implements `Send` for a type using `unsafe impl`.
///
/// Generics are supported: type and lifetime parameters and where clauses
/// are carried through to the generated impl.
///
/// # Safety
///
/// This macro bypasses Rust's automatic `Send` verification. You must ensure
/// that your type is actually safe to transfer across threads. Improper use can
/// cause data races and undefined behavior.
///
/// Note: if every field is already `Send`, the type is automatically `Send`
/// and this derive is redundant.
///
/// Note: the generated impl is exempt from `#![forbid(unsafe_code)]`, like
/// all derive-generated code — the guard does not flag it. Audit the types
/// you apply this derive to yourself.
///
/// # Example
///
/// ```rust
/// use libu::Send;
/// use std::rc::Rc;
///
/// // WARNING: Only use if you know this is safe!
/// // `Rc` is neither `Send` nor `Sync`; the derive forces it.
/// #[derive(Send)]
/// struct MyType {
///   marker: Rc<()>,
/// }
/// ```
#[proc_macro_derive(Send)]
pub fn derive_send(input: TokenStream) -> TokenStream {
  send::expand(input.into()).into()
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
/// Receivers may be written with or without a leading `&`.
///
/// # Syntax
///
/// ```rust
/// use libu_derive::select;
/// use flume::unbounded;
///
/// let (tx, rx) = unbounded();
/// tx.send(42).unwrap();
///
/// select! [
///   &rx => |msg| { assert_eq!(msg, Ok(42)); },
/// ];
/// ```
///
/// Each handler receives the `Result<T, flume::RecvError>` produced by
/// `flume::Selector::recv` — check it for a disconnected channel:
///
/// ```rust
/// use libu_derive::select;
/// use flume::unbounded;
///
/// let (tx, rx) = unbounded();
/// tx.send(1).unwrap();
/// drop(tx); // disconnect after the pending message
///
/// select! [
///   &rx => |msg| {
///     match msg {
///       Ok(m) => assert_eq!(m, 1),
///       Err(_) => panic!("disconnected before the message arrived"),
///     }
///   },
/// ];
/// ```
///
/// At least one arm is required, and arms must be separated by commas.
/// The flume crate path defaults to `::flume` and can be overridden with a
/// leading path: `select!(::libu::dependency::flume; &rx => h, ...)`.
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
