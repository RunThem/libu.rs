use proc_macro2::TokenStream;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
  let input = match syn::parse2::<syn::DeriveInput>(input) {
    Ok(input) => input,
    Err(err) => return err.to_compile_error(),
  };
  let ident = input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  // SAFETY: The user has explicitly opted into this unsafe implementation.
  // They must ensure this type is actually safe to share across threads.
  quote::quote! {
    #[automatically_derived]
    unsafe impl #impl_generics ::core::marker::Sync for #ident #ty_generics #where_clause {}
  }
  .into()
}

#[cfg(test)]
mod tests {
  use super::*;
  use quote::ToTokens;
  use syn::parse_quote;

  /// Run the codegen and return the generated tokens.
  fn expand(input: syn::DeriveInput) -> String {
    super::expand(input.to_token_stream()).to_string()
  }

  // The end-to-end pass cases live in tests/ui/pass/ (e.g. send_sync.rs);
  // they are compiled and run once by builder::tests::trybuild_compile_pass,
  // which globs the shared fixture directory.

  // -------------------------------------------------------------------------
  // Input shapes: the codegen only reads the type name, so every item shape
  // must produce the same `unsafe impl` regardless of its data.
  // -------------------------------------------------------------------------

  #[test]
  fn named_struct() {
    let input: syn::DeriveInput = parse_quote! {
      struct Server {
        name: String,
      }
    };
    let out = expand(input);

    assert!(
      out.contains("unsafe impl :: core :: marker :: Sync for Server { }"),
      "{out}"
    );
    // Derive convention: the impl carries #[automatically_derived].
    assert!(out.contains("automatically_derived"), "{out}");
    // Only the one impl — nothing else leaks into the output.
    assert_eq!(
      out.matches("unsafe impl :: core :: marker :: Sync").count(),
      1,
      "{out}"
    );
    // The Sync codegen never mentions Send.
    assert!(!out.contains("Send"), "{out}");
  }

  #[test]
  fn tuple_struct() {
    let input: syn::DeriveInput = parse_quote! {
      struct Point(f32, f32);
    };
    let out = expand(input);

    assert!(
      out.contains("unsafe impl :: core :: marker :: Sync for Point { }"),
      "{out}"
    );
  }

  #[test]
  fn unit_struct() {
    let input: syn::DeriveInput = parse_quote! {
      struct Marker;
    };
    let out = expand(input);

    assert!(
      out.contains("unsafe impl :: core :: marker :: Sync for Marker { }"),
      "{out}"
    );
  }

  #[test]
  fn enum_input() {
    let input: syn::DeriveInput = parse_quote! {
      enum Color {
        Red,
        Green,
        Blue,
      }
    };
    let out = expand(input);

    assert!(
      out.contains("unsafe impl :: core :: marker :: Sync for Color { }"),
      "{out}"
    );
  }

  #[test]
  fn union_input() {
    let input: syn::DeriveInput = parse_quote! {
      union Bits {
        f: u32,
        i: i32,
      }
    };
    let out = expand(input);

    assert!(
      out.contains("unsafe impl :: core :: marker :: Sync for Bits { }"),
      "{out}"
    );
  }

  #[test]
  fn raw_identifier() {
    let input: syn::DeriveInput = parse_quote! {
      struct r#type {
        x: u32,
      }
    };
    let out = expand(input);

    // The raw ident must survive the round trip.
    assert!(
      out.contains("unsafe impl :: core :: marker :: Sync for r#type { }"),
      "{out}"
    );
  }

  // -------------------------------------------------------------------------
  // Generics: type and lifetime parameters, where clauses and const
  // generics are carried through to the impl head.
  // -------------------------------------------------------------------------

  #[test]
  fn generic_type_param() {
    let input: syn::DeriveInput = parse_quote! {
      struct Foo<T>(T);
    };
    let out = expand(input);

    assert!(
      out.contains("unsafe impl < T > :: core :: marker :: Sync for Foo < T > { }"),
      "{out}"
    );
  }

  #[test]
  fn generic_lifetime() {
    let input: syn::DeriveInput = parse_quote! {
      struct Foo<'a>(&'a str);
    };
    let out = expand(input);

    assert!(
      out.contains("unsafe impl < 'a > :: core :: marker :: Sync for Foo < 'a > { }"),
      "{out}"
    );
  }

  #[test]
  fn generic_where_clause() {
    let input: syn::DeriveInput = parse_quote! {
      struct Foo<T>(T)
      where
        T: Default;
    };
    let out = expand(input);

    assert!(
      out.contains(
        "unsafe impl < T > :: core :: marker :: Sync for Foo < T > where T : Default { }"
      ),
      "{out}"
    );
  }

  #[test]
  fn generic_const() {
    let input: syn::DeriveInput = parse_quote! {
      struct Arr<const N: usize>([u8; N]);
    };
    let out = expand(input);

    assert!(
      out.contains("unsafe impl < const N : usize > :: core :: marker :: Sync for Arr < N > { }"),
      "{out}"
    );
  }
}
