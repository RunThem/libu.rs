use darling::{ast, util};
use proc_macro2::{Ident, TokenStream as Ts};
use quote::quote;
use syn::{Attribute, Error, PathArguments, Type, TypePath, Visibility};

/// Rust strict and reserved keywords, which proc-macro2 accepts as
/// identifiers but the generated code would never compile with.
const RUST_KEYWORDS: &[&str] = &[
  "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
  "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
  "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
  "use", "where", "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv",
  "typeof", "unsized", "virtual", "yield", "try", "union",
];

#[derive(Debug, darling::FromField)]
#[darling(attributes(builder), forward_attrs(allow, doc, cfg))]
pub(crate) struct Field {
  /// Field identifier
  pub(crate) ident: Option<Ident>,
  /// Field type
  pub(crate) ty: Type,
  /// Field attributes (supports allow/doc/cfg)
  pub(crate) attrs: Vec<Attribute>,

  /// Accept `impl Into<T>` in setter method.
  ///
  /// Defaults to on for `String` fields (so both `&str` and `String`
  /// work); `#[builder(into)]` forces it on for any field,
  /// `#[builder(into = false)]` forces it off.
  #[darling(default)]
  pub(crate) into: Option<bool>,

  /// Setter method name prefix. Defaults to `with_`; an empty string
  /// disables the prefix (`#[builder(prefix = "")]`).
  #[darling(default)]
  pub(crate) prefix: Option<String>,

  /// Field is kept in the builder, but no setter is generated.
  /// Useful for internal state initialized via `#[builder(default = ...)]`.
  #[darling(default)]
  pub(crate) skip: bool,

  /// Setter is generated as `pub(crate)` instead of `pub`.
  #[darling(default)]
  pub(crate) private: bool,

  /// Field must be initialized (panics if not set)
  #[darling(default)]
  pub(crate) must: bool,

  /// Default value expression used when the field is not set.
  ///
  /// Takes precedence over `Default::default()`; useful for enums that don't
  /// implement `Default` (e.g. `#[builder(default = Mode::Test)]`).
  #[darling(default)]
  pub(crate) default: Option<syn::Expr>,
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(supports(struct_named), forward_attrs(allow, doc, cfg))]
pub(crate) struct BuilderDeriveInput {
  pub(crate) vis: Visibility,
  /// Struct identifier
  pub(crate) ident: Ident,
  /// Struct attributes
  pub(crate) attrs: Vec<Attribute>,
  /// Struct fields
  pub(crate) data: ast::Data<util::Ignored, Field>,
  /// Generic parameters
  pub(crate) generics: ast::Generics<syn::GenericParam>,
}

impl quote::ToTokens for BuilderDeriveInput {
  fn to_tokens(&self, tokens: &mut Ts) {
    let mut fields = vec![];
    let mut defaults = vec![];
    let mut methods = vec![];
    let mut build = vec![];

    let BuilderDeriveInput {
      vis,
      ident,
      attrs,
      data,
      generics,
    } = self;

    let builder_ident = Ident::new(&format!("{ident}Builder"), ident.span());
    let struct_fields = data.as_ref().take_struct().unwrap();

    // A plain field's build code requires `Default` on its type; route that
    // requirement through a hidden trait so a missing impl produces a helpful
    // diagnostic (via on_unimplemented) instead of a bare E0277.
    let has_plain_fields = struct_fields.iter().any(|f| {
      let (ty, is_option) = get_option_inner_type(&f.ty);
      !is_option && !f.must && f.default.is_none()
    });
    let default_trait = Ident::new(&format!("__{ident}BuilderDefault"), ident.span());

    // Every generated setter name (including the _opt variants), used to catch
    // cross-field collisions like `x: Option<u32>` + `x_opt: u32`.
    let mut setter_names = vec![];

    for field in struct_fields {
      let Field {
        ident, ty, attrs, ..
      } = field;

      let ident = ident.as_ref().unwrap();
      let (ty, is_option) = get_option_inner_type(ty);

      if field.must && field.skip {
        tokens.extend(
          Error::new_spanned(
            ident,
            "`#[builder(must)]` and `#[builder(skip)]` cannot be combined: \
             skip removes the setter, so the field could never be initialized",
          )
          .to_compile_error(),
        );
        return;
      }

      if field.must && field.default.is_some() {
        tokens.extend(
          Error::new_spanned(
            ident,
            "`#[builder(must)]` and `#[builder(default = ...)]` cannot be combined: \
             `must` panics when unset, which makes the default unreachable",
          )
          .to_compile_error(),
        );
        return;
      }

      fields.push(quote! {
        #(#attrs)*
        #ident: std::option::Option<#ty>
      });

      defaults.push(quote! (#ident: std::option::Option::None));

      let setter_name = format!("{}{ident}", field.prefix.as_deref().unwrap_or("with_"));
      // Validate before constructing: the XID check in syn's Ident parse
      // matches rustc (including Unicode identifiers), and rebuilding with
      // the field's span keeps error locations on the field. syn rejects
      // keywords outright, so the keyword table only refines the message.
      let setter_ident = if syn::parse_str::<Ident>(&setter_name).is_ok() {
        Ident::new(&setter_name, ident.span())
      } else if RUST_KEYWORDS.contains(&setter_name.as_str()) {
        tokens.extend(
          Error::new_spanned(
            ident,
            format!(
              "`{setter_name}` is not a valid setter name (reserved keyword); \
               use `#[builder(prefix = ...)]` or rename the field"
            ),
          )
          .to_compile_error(),
        );
        return;
      } else {
        tokens.extend(
          Error::new_spanned(
            ident,
            format!("`{setter_name}` is not a valid setter name (from `#[builder(prefix = ...)]`)"),
          )
          .to_compile_error(),
        );
        return;
      };

      if !field.skip && (setter_name == "build" || setter_name == "default") {
        tokens.extend(
          Error::new_spanned(
            ident,
            format!(
              "setter name `{setter_name}` conflicts with a method generated by the builder; \
               use `#[builder(prefix = ...)]` or rename the field"
            ),
          )
          .to_compile_error(),
        );
        return;
      }

      if !field.skip && setter_names.contains(&setter_name) {
        tokens.extend(
          Error::new_spanned(
            ident,
            format!(
              "setter name `{setter_name}` is generated for multiple fields; \
               rename one of them or adjust `#[builder(prefix = ...)]`"
            ),
          )
          .to_compile_error(),
        );
        return;
      }
      if !field.skip {
        setter_names.push(setter_name.clone());
      }

      if is_option && field.must {
        tokens.extend(
          Error::new_spanned(
            ident,
            "`#[builder(must)]` cannot be used on `Option<T>` fields: the setter already \
             takes `T`, and an unset field is `None`; drop the attribute or use a \
             non-Option field type",
          )
          .to_compile_error(),
        );
        return;
      }

      if is_option && field.default.is_some() {
        let hint = if field.skip {
          "drop the attribute".to_string()
        } else {
          format!("pass the value explicitly via `{setter_name}_opt(Some(...))`")
        };
        tokens.extend(
          Error::new_spanned(
            ident,
            format!("`#[builder(default = ...)]` cannot be used on `Option<T>` fields: {hint}",),
          )
          .to_compile_error(),
        );
        return;
      }

      let setter_vis = if field.private {
        quote!(pub(crate))
      } else {
        quote!(pub)
      };

      let into = field.into.unwrap_or(is_string(ty));

      let method = if field.skip {
        quote!()
      } else if into {
        quote! {
          #setter_vis fn #setter_ident(mut self, #ident: impl Into<#ty>) -> Self {
            self.#ident = std::option::Option::Some(#ident.into());
            self
          }
        }
      } else {
        quote! {
          #setter_vis fn #setter_ident(mut self, #ident: #ty) -> Self {
            self.#ident = std::option::Option::Some(#ident);
            self
          }
        }
      };

      methods.push(method);

      // Option<T> fields additionally get a with_<field>_opt setter that
      // accepts the Option<T> itself, so a pre-built Option can be passed
      // straight through without an if-let dance.
      if is_option && !field.skip {
        let opt_name = format!("{}_opt", setter_ident);
        if setter_names.contains(&opt_name) {
          tokens.extend(
            Error::new_spanned(
              ident,
              format!(
                "setter name `{opt_name}` is generated for multiple fields; \
                 rename one of them or adjust `#[builder(prefix = ...)]`"
              ),
            )
            .to_compile_error(),
          );
          return;
        }
        setter_names.push(opt_name.clone());

        let opt_ident = Ident::new(&opt_name, ident.span());
        let opt_param = Ident::new(&format!("{ident}_opt"), ident.span());
        methods.push(quote! {
          #setter_vis fn #opt_ident(mut self, #opt_param: std::option::Option<#ty>) -> Self {
            self.#ident = #opt_param;
            self
          }
        });
      }

      if is_option {
        build.push(quote! (#ident: self.#ident));
      } else if field.must {
        let msg = format!("Field '{}' must be initialized", ident);
        build.push(quote! (#ident: self.#ident.expect(#msg)));
      } else if let Some(default) = &field.default {
        // unwrap_or_else keeps the expression lazy: it only runs when the
        // field was not set.
        build.push(quote! (#ident: self.#ident.unwrap_or_else(|| #default)));
      } else {
        build.push(quote! {
          #ident: self.#ident.unwrap_or_else(|| <#ty as #default_trait>::__builder_default())
        });
      }
    }

    let where_clause = &generics.where_clause;
    let generics_params = &generics.params;
    let struct_params = quote! (<#(#generics_params), *>);

    // Impl-head declaration: keep bounds but drop defaults, which are not
    // allowed on impls (`struct S<T = i32>`, `struct S<const N: usize = 4>`).
    let mut impl_generics = generics.params.clone();
    impl_generics.iter_mut().for_each(|t| match t {
      syn::GenericParam::Type(t) => {
        t.eq_token = None;
        t.default = None;
      }
      syn::GenericParam::Const(c) => {
        c.eq_token = None;
        c.default = None;
      }
      syn::GenericParam::Lifetime(_) => {}
    });
    let impl_generics = quote! (<#(#impl_generics), *>);

    // Type references (builder self type, build()/builder() return types):
    // bare idents, so const params expand to `N` rather than
    // `const N: usize`.
    let type_refs: Vec<_> = generics
      .params
      .iter()
      .map(|t| match t {
        syn::GenericParam::Type(t) => {
          let i = &t.ident;
          quote! (#i)
        }
        syn::GenericParam::Lifetime(l) => {
          let l = &l.lifetime;
          quote! (#l)
        }
        syn::GenericParam::Const(c) => {
          let i = &c.ident;
          quote! (#i)
        }
      })
      .collect();
    let type_refs = quote! (<#(#type_refs), *>);

    let default_helper = if has_plain_fields {
      quote! {
        #[doc(hidden)]
        #[allow(dead_code)]
        #[diagnostic::on_unimplemented(
          message = "field type must implement `Default`",
          note = "add #[builder(must)] or #[builder(default = ...)] to the field",
          label = "`{Self}` does not implement `Default`"
        )]
        trait #default_trait {
          fn __builder_default() -> Self;
        }

        impl<T: ::std::default::Default> #default_trait for T {
          fn __builder_default() -> Self {
            ::std::default::Default::default()
          }
        }
      }
    } else {
      quote!()
    };

    tokens.extend(quote! {
      #default_helper

      #(#attrs)*
      #vis struct #builder_ident #struct_params #where_clause {
        #(#fields),*
      }

      // Hand-written instead of #[derive(Default)]: every field is Option<..>,
      // so defaulting to all-None needs no bounds on the type parameters.
      impl #impl_generics #builder_ident #type_refs #where_clause {
        fn default() -> Self {
          Self { #(#defaults),* }
        }
      }

      impl #impl_generics #builder_ident #type_refs #where_clause {
        #(#methods)*

        pub fn build(self) -> #ident #type_refs {
          #ident { #(#build),* }
        }
      }

      impl #impl_generics #ident #type_refs #where_clause {
        pub fn builder() -> #builder_ident #type_refs {
          #builder_ident::default()
        }
      }
    });
  }
}

fn get_option_inner_type(ty: &Type) -> (&Type, bool) {
  if let Type::Path(TypePath { path, .. }) = ty {
    if is_std_named(ty, "Option", "option")
      && let Some(segment) = path.segments.last()
      && let PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }) =
        &segment.arguments
      && let Some(syn::GenericArgument::Type(inner_ty)) = args.first()
    {
      return (inner_ty, true);
    }
  }

  (ty, false)
}

fn is_string(ty: &Type) -> bool {
  is_std_named(ty, "String", "string")
}

/// True when `ty` names the std type `module::name` — either bare (`Option`,
/// `String`, as resolved through the prelude) or spelled with the full
/// `std`/`core`/`alloc` path. Custom types under other paths (e.g.
/// `fake::Option<T>`) do not match.
fn is_std_named(ty: &Type, name: &str, module: &str) -> bool {
  if let Type::Path(TypePath { qself: None, path }) = ty {
    let idents: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    if idents.len() == 1 {
      return idents[0] == name;
    }
    if idents.len() >= 3 {
      return (idents[0] == "std" || idents[0] == "core" || idents[0] == "alloc")
        && idents[1] == module
        && idents[2] == name;
    }
  }

  false
}

#[cfg(test)]
mod tests {
  use super::*;
  use darling::FromDeriveInput;
  use quote::ToTokens;
  use syn::parse_quote;

  /// Run the full pipeline (parse + codegen) and return the generated tokens.
  fn expand(input: syn::DeriveInput) -> String {
    BuilderDeriveInput::from_derive_input(&input)
      .unwrap()
      .to_token_stream()
      .to_string()
  }

  // Compile-error diagnostics live in tests/ui/ as trybuild snapshots: the
  // compile-fail cases pin the exact error text and span, and the pass case
  // compiles and runs the generated code. Error-path assertions belong there,
  // not here, so that wording changes touch only the snapshots.
  #[test]
  fn trybuild_compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
  }

  #[test]
  fn trybuild_compile_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/*.rs");
  }

  // -------------------------------------------------------------------------
  // Setter naming: with_ prefix, custom prefix, no prefix, validation
  // -------------------------------------------------------------------------

  #[test]
  fn default_with_prefix() {
    let input: syn::DeriveInput = parse_quote! {
      struct Server {
        name: String,
        #[builder(prefix = "set_")]
        port: u16,
        #[builder(prefix = "")]
        host: String,
        #[builder(skip, default = "default_region".to_string())]
        region: String,
        #[builder(private)]
        secret: String,
        #[builder(must)]
        mode: Mode,
        #[builder(default = Mode::Test)]
        fallback: Mode,
        timeout: Option<u64>,
        #[builder(into)]
        count: u32,
      }
    };
    let out = expand(input);

    // Default with_ prefix.
    assert!(out.contains("fn with_name"), "{out}");
    assert!(out.contains("fn with_timeout"), "{out}");
    // prefix = "set_" overrides.
    assert!(out.contains("fn set_port"), "{out}");
    assert!(!out.contains("fn with_port"), "{out}");
    // prefix = "" disables the prefix.
    assert!(out.contains("fn host"), "{out}");
    // skip: no setter at all.
    assert!(!out.contains("fn with_region"), "{out}");
    // private: pub(crate) setter.
    assert!(out.contains("pub (crate) fn with_secret"), "{out}");
    // Option fields get a second _opt setter; others do not.
    assert!(out.contains("fn with_timeout_opt"), "{out}");
    assert!(!out.contains("fn with_port_opt"), "{out}");
    // String fields default to impl Into<String>; #[builder(into)] opts in.
    assert!(out.contains("impl Into < String >"), "{out}");
    assert!(
      out.contains("fn with_count (mut self , count : impl Into < u32 >)"),
      "{out}"
    );
    // must: expect with the field name baked in.
    assert!(
      out.contains("expect (\"Field 'mode' must be initialized\")"),
      "{out}"
    );
    // default = expr: lazy unwrap_or_else, not eager unwrap_or.
    assert!(out.contains("unwrap_or_else (|| Mode :: Test)"), "{out}");
    // Option fields pass through without unwrapping.
    assert!(out.contains("timeout : self . timeout"), "{out}");
    // Hand-written Default impl instead of #[derive(Default)].
    assert!(!out.contains("derive (Default)"), "{out}");
    // Helper trait with the per-struct name and the diagnostic attribute.
    assert!(out.contains("on_unimplemented"), "{out}");
    assert!(out.contains("__ServerBuilderDefault"), "{out}");
  }

  // -------------------------------------------------------------------------
  // Field options: skip, private, into, must, default
  // -------------------------------------------------------------------------

  #[test]
  fn parses_field_attributes() {
    let input: syn::DeriveInput = parse_quote! {
      struct S {
        a: u32,
        #[builder(skip, default = 1)]
        b: u32,
        #[builder(must)]
        c: u32,
        #[builder(prefix = "set_")]
        d: u32,
        #[builder(into = false)]
        e: String,
        #[builder(private)]
        f: u32,
        #[builder(into)]
        g: u32,
      }
    };
    let parsed = BuilderDeriveInput::from_derive_input(&input).unwrap();
    let fields: Vec<&Field> = parsed
      .data
      .as_ref()
      .take_struct()
      .unwrap()
      .iter()
      .copied()
      .collect();

    assert!(!fields[0].skip && !fields[0].must && fields[0].default.is_none());
    assert!(fields[1].skip && fields[1].default.is_some());
    assert!(fields[2].must);
    assert_eq!(fields[3].prefix.as_deref(), Some("set_"));
    assert_eq!(fields[4].into, Some(false));
    assert!(fields[5].private);
    assert_eq!(fields[6].into, Some(true));
  }

  #[test]
  fn into_false_keeps_owned_param() {
    let input: syn::DeriveInput = parse_quote! {
      struct S {
        #[builder(into = false)]
        tag: String,
      }
    };
    let out = expand(input);

    assert!(
      out.contains("fn with_tag (mut self , tag : String)"),
      "{out}"
    );
    assert!(!out.contains("impl Into"), "{out}");
  }

  #[test]
  fn must_with_into() {
    let input: syn::DeriveInput = parse_quote! {
      struct S {
        #[builder(must, into)]
        len: u64,
      }
    };
    let out = expand(input);

    assert!(out.contains("impl Into < u64 >"), "{out}");
    assert!(out.contains("expect"), "{out}");
  }

  #[test]
  fn default_expression_is_lazy() {
    let input: syn::DeriveInput = parse_quote! {
      struct S {
        #[builder(default = counter.fetch_add(1, Ordering::Relaxed))]
        n: u32,
      }
    };
    let out = expand(input);

    assert!(out.contains("unwrap_or_else"), "{out}");
    assert!(!out.contains("unwrap_or ("), "{out}");
  }

  #[test]
  fn no_helper_trait_without_plain_fields() {
    let input: syn::DeriveInput = parse_quote! {
      struct S {
        #[builder(must)]
        x: u32,
        y: Option<u32>,
      }
    };
    let out = expand(input);

    assert!(!out.contains("on_unimplemented"), "{out}");
  }

  // -------------------------------------------------------------------------
  // Option fields: dual setters, nesting, skip/private interplay
  // -------------------------------------------------------------------------

  #[test]
  fn nested_option_strips_one_layer() {
    let input: syn::DeriveInput = parse_quote! {
      struct Nested {
        deep: Option<Option<u64>>,
      }
    };
    let out = expand(input);

    assert!(
      out.contains("fn with_deep (mut self , deep : Option < u64 >)"),
      "{out}"
    );
    assert!(
      out.contains(
        "fn with_deep_opt (mut self , deep_opt : std :: option :: Option < Option < u64 > >)"
      ),
      "{out}"
    );
    assert!(out.contains("deep : self . deep"), "{out}");
  }

  #[test]
  fn skip_option_has_no_setter() {
    let input: syn::DeriveInput = parse_quote! {
      struct S {
        #[builder(skip)]
        flag: Option<bool>,
      }
    };
    let out = expand(input);

    assert!(!out.contains("fn with_flag"), "{out}");
    assert!(!out.contains("fn with_flag_opt"), "{out}");
    assert!(out.contains("flag : self . flag"), "{out}");
  }

  #[test]
  fn private_option_both_setters() {
    let input: syn::DeriveInput = parse_quote! {
      struct S {
        #[builder(private)]
        token: Option<u32>,
      }
    };
    let out = expand(input);

    assert!(out.contains("pub (crate) fn with_token"), "{out}");
    assert!(out.contains("pub (crate) fn with_token_opt"), "{out}");
  }

  #[test]
  fn custom_option_type_not_misdetected() {
    let input: syn::DeriveInput = parse_quote! {
      struct F {
        #[builder(must)]
        f: fake::Option<u32>,
      }
    };
    let out = expand(input);

    // fake::Option<T> keeps its own type: no stripping, no _opt setter.
    assert!(
      out.contains("fn with_f (mut self , f : fake :: Option < u32 >)"),
      "{out}"
    );
    assert!(!out.contains("fn with_f_opt"), "{out}");
  }

  // -------------------------------------------------------------------------
  // Generics: lifetimes, bounds, where clauses, const generics, defaults
  // -------------------------------------------------------------------------

  #[test]
  fn lifetimes_and_where_clauses() {
    let input: syn::DeriveInput = parse_quote! {
      struct WData<'a, T>
      where
        T: Default,
      {
        text: &'a str,
        value: T,
      }
    };
    let out = expand(input);

    assert!(out.contains("WDataBuilder < 'a , T >"), "{out}");
    assert!(out.contains("where T : Default"), "{out}");
  }

  #[test]
  fn const_generics() {
    let input: syn::DeriveInput = parse_quote! {
      struct Arr<const N: usize> {
        #[builder(must)]
        data: [u8; N],
      }
    };
    let out = expand(input);

    // impl head keeps the full declaration, type references use the bare ident.
    assert!(
      out.contains("impl < const N : usize > ArrBuilder < N >"),
      "{out}"
    );
    assert!(
      !out.contains("impl < const N : usize > ArrBuilder < const N : usize >"),
      "{out}"
    );
  }

  #[test]
  fn default_type_param() {
    let input: syn::DeriveInput = parse_quote! {
      struct Def<T = i32> {
        #[builder(must)]
        val: T,
      }
    };
    let out = expand(input);

    // struct declaration keeps the default; the impl head must not repeat it.
    assert!(out.contains("struct DefBuilder < T = i32 >"), "{out}");
    assert_eq!(out.matches("T = i32").count(), 1, "{out}");
  }

  #[test]
  fn empty_struct() {
    let input: syn::DeriveInput = parse_quote! { struct Empty {} };
    let out = expand(input);

    assert!(out.contains("struct EmptyBuilder"), "{out}");
    assert!(out.contains("fn build (self)"), "{out}");
  }

  #[test]
  fn helper_trait_is_per_struct() {
    let a: syn::DeriveInput = parse_quote! { struct A { x: u32 } };
    let b: syn::DeriveInput = parse_quote! { struct B { y: u32 } };
    let out_a = expand(a);
    let out_b = expand(b);

    assert!(out_a.contains("__ABuilderDefault"), "{out_a}");
    assert!(out_b.contains("__BBuilderDefault"), "{out_b}");
    assert!(!out_a.contains("__BBuilderDefault"), "{out_a}");
  }

  // -------------------------------------------------------------------------
  // Std path detection helpers
  // -------------------------------------------------------------------------

  #[test]
  fn detects_std_option_and_string() {
    let opt: syn::Type = parse_quote!(Option<u32>);
    let (inner, is_opt) = get_option_inner_type(&opt);
    assert!(is_opt);
    // extra-traits: the inner type can be compared as an AST node.
    assert_eq!(inner, &parse_quote!(u32));

    let full: syn::Type = parse_quote!(::std::option::Option<u32>);
    assert_eq!(get_option_inner_type(&full).1, true);

    let core: syn::Type = parse_quote!(core::option::Option<u32>);
    assert_eq!(get_option_inner_type(&core).1, true);

    let fake: syn::Type = parse_quote!(fake::Option<u32>);
    assert_eq!(get_option_inner_type(&fake).1, false);

    let s: syn::Type = parse_quote!(String);
    assert!(is_string(&s));

    let s_full: syn::Type = parse_quote!(std::string::String);
    assert!(is_string(&s_full));

    let s_custom: syn::Type = parse_quote!(MyString);
    assert!(!is_string(&s_custom));

    let s_fake: syn::Type = parse_quote!(fake::String);
    assert!(!is_string(&s_fake));
  }

  #[test]
  fn parses_field_types() {
    let input: syn::DeriveInput = parse_quote! {
      struct S {
        a: String,
        b: Option<u64>,
      }
    };
    let parsed = BuilderDeriveInput::from_derive_input(&input).unwrap();
    let fields: Vec<&Field> = parsed
      .data
      .as_ref()
      .take_struct()
      .unwrap()
      .iter()
      .copied()
      .collect();

    assert_eq!(fields[0].ty, parse_quote!(String));
    assert_eq!(fields[1].ty, parse_quote!(Option<u64>));
  }
}
