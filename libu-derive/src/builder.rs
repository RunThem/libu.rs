use darling::{ast, util};
use proc_macro2::{Ident, TokenStream as Ts};
use quote::quote;
use syn::{Attribute, PathArguments, Type, TypePath, Visibility};

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

    for field in struct_fields {
      let Field {
        ident, ty, attrs, ..
      } = field;

      let ident = ident.as_ref().unwrap();
      let (ty, is_option) = get_option_inner_type(ty);

      fields.push(quote! {
        #(#attrs)*
        #ident: std::option::Option<#ty>
      });

      let setter_ident = Ident::new(
        &format!("{}{ident}", field.prefix.as_deref().unwrap_or("with_")),
        ident.span(),
      );

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
        let opt_ident = Ident::new(&format!("{}_opt", setter_ident), ident.span());
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
        build.push(quote! (#ident: self.#ident.unwrap_or(#default)));
      } else {
        build.push(quote! (#ident: self.#ident.unwrap_or_default()));
      }
    }

    let mut generics_impl_params = generics.params.clone();
    generics_impl_params.iter_mut().for_each(|t| {
      if let syn::GenericParam::Type(t) = t {
        t.bounds.clear();
        t.eq_token = None;
        t.default = None;
      }
    });

    let where_clause = &generics.where_clause;
    let generics_params = &generics.params;
    let generics_params = quote! (<#(#generics_params), *>);
    let generics_impl_params = quote! (<#(#generics_impl_params), *>);

    tokens.extend(quote! {
      #[derive(Default)]
      #(#attrs)*
      #vis struct #builder_ident #generics_params #where_clause {
        #(#fields),*
      }

      impl #generics_params #builder_ident #generics_impl_params #where_clause {
        #(#methods)*

        pub fn build(self) -> #ident #generics_impl_params {
          #ident { #(#build),* }
        }
      }

      impl #generics_params #ident #generics_impl_params #where_clause {
        pub fn builder() -> #builder_ident #generics_impl_params {
          #builder_ident::default()
        }
      }
    });
  }
}

fn get_option_inner_type(ty: &Type) -> (&Type, bool) {
  if let Type::Path(TypePath { path, .. }) = ty {
    if let Some(segment) = path.segments.last()
      && segment.ident == "Option"
    {
      if let PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }) =
        &segment.arguments
      {
        if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
          return (inner_ty, true);
        }
      }
    }
  }

  (ty, false)
}

fn is_string(ty: &Type) -> bool {
  if let Type::Path(TypePath { path, .. }) = ty
    && let Some(segment) = path.segments.last()
  {
    return segment.ident == "String";
  }

  false
}