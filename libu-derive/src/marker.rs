//! Shared `unsafe impl ::core::marker::X` codegen for the `Send`/`Sync` derives.
//!
//! Both derives emit the identical impl shape apart from the marker trait:
//! `#[automatically_derived] unsafe impl <generics> ::core::marker::{Send|Sync}
//! for <ident> <ty_generics> <where_clause> {}`. `send.rs` and `sync.rs` are
//! one-line forwards to [`expand`]; the per-trait safety contract lives in the
//! `Send`/`Sync` derive docs at the crate root.

use proc_macro2::{Ident, Span, TokenStream};

/// Generate `unsafe impl <marker> for <ident>` carrying the type's generics,
/// lifetimes and where clause through to the impl head.
///
/// # SAFETY
///
/// The emitted impl is `unsafe` intentionally — the caller (the `Send`/`Sync`
/// derives) must guarantee the type really is safe to transfer/share across
/// threads. Keep it `unsafe`; do not make it a safe impl.
pub(crate) fn expand(input: TokenStream, marker: &str) -> TokenStream {
  let input = match syn::parse2::<syn::DeriveInput>(input) {
    Ok(input) => input,
    Err(err) => return err.to_compile_error(),
  };
  let ident = input.ident;
  let marker = Ident::new(marker, Span::call_site());
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  quote::quote! {
    #[automatically_derived]
    unsafe impl #impl_generics ::core::marker::#marker for #ident #ty_generics #where_clause {}
  }
  .into()
}

#[cfg(test)]
mod tests {
  use quote::ToTokens;
  use syn::parse_quote;

  /// Run `input` through one marker and return the generated tokens.
  fn expand(input: syn::DeriveInput, marker: &str) -> String {
    super::expand(input.to_token_stream(), marker).to_string()
  }

  /// Run a shape through both markers, asserting each produces exactly the one
  /// correct impl (`tail` is everything after `:: core :: marker :: <marker>`).
  fn check(input: syn::DeriveInput, tail: &str) {
    for marker in ["Send", "Sync"] {
      let out = expand(input.clone(), marker);

      let needle = format!(":: core :: marker :: {marker} {tail}");
      assert!(out.contains(&needle), "{out}");
      // Derive convention: the impl carries #[automatically_derived].
      assert!(out.contains("automatically_derived"), "{out}");
      // Only the one impl — nothing else leaks into the output.
      assert_eq!(
        out
          .matches(&format!(":: core :: marker :: {marker}"))
          .count(),
        1,
        "{out}"
      );
      // Each codegen only ever mentions its own marker.
      let other = if marker == "Send" { "Sync" } else { "Send" };
      assert!(!out.contains(other), "{out}");
    }
  }

  // -------------------------------------------------------------------------
  // Input shapes: the codegen only reads the type name, so every item shape
  // must produce the same `unsafe impl` regardless of its data.
  // -------------------------------------------------------------------------

  #[test]
  fn named_struct() {
    check(
      parse_quote! {
        struct Server {
          name: String,
        }
      },
      "for Server { }",
    );
  }

  #[test]
  fn tuple_struct() {
    check(parse_quote! { struct Point(f32, f32); }, "for Point { }");
  }

  #[test]
  fn unit_struct() {
    check(parse_quote! { struct Marker; }, "for Marker { }");
  }

  #[test]
  fn enum_input() {
    check(
      parse_quote! {
        enum Color {
          Red,
          Green,
          Blue,
        }
      },
      "for Color { }",
    );
  }

  #[test]
  fn union_input() {
    check(
      parse_quote! {
        union Bits {
          f: u32,
          i: i32,
        }
      },
      "for Bits { }",
    );
  }

  #[test]
  fn raw_identifier() {
    check(
      parse_quote! {
        struct r#type {
          x: u32,
        }
      },
      "for r#type { }",
    );
  }

  // -------------------------------------------------------------------------
  // Generics: type and lifetime parameters, where clauses and const generics
  // are carried through to the impl head.
  // -------------------------------------------------------------------------

  #[test]
  fn generic_type_param() {
    check(parse_quote! { struct Foo<T>(T); }, "for Foo < T > { }");
  }

  #[test]
  fn generic_lifetime() {
    check(
      parse_quote! { struct Foo<'a>(&'a str); },
      "for Foo < 'a > { }",
    );
  }

  #[test]
  fn generic_where_clause() {
    check(
      parse_quote! {
        struct Foo<T>(T)
        where
          T: Default;
      },
      "for Foo < T > where T : Default { }",
    );
  }

  #[test]
  fn generic_const() {
    check(
      parse_quote! {
        struct Arr<const N: usize>([u8; N]);
      },
      "for Arr < N > { }",
    );
  }
}
