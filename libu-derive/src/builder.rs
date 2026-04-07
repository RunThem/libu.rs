use darling::{ast, util};
use proc_macro2::{Ident, TokenStream as Ts};
use quote::quote;
use syn::{Attribute, Error, PathArguments, Type, TypePath, Visibility};

#[derive(Debug, darling::FromField)]
#[darling(attributes(builder), forward_attrs(allow, doc, cfg))]
pub(crate) struct Field {
  /// Field identifier
  pub(crate) ident: Option<Ident>,
  /// Field type
  pub(crate) ty: Type,
  /// Field attributes (supports allow/doc/cfg)
  pub(crate) attrs: Vec<Attribute>,

  /// Accept `impl Into<T>` in setter method
  #[darling(default)]
  pub(crate) into: bool,

  /// Field must be initialized (panics if not set)
  #[darling(default)]
  pub(crate) must: bool,
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
    let mut init = vec![];
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

    if !data.is_struct() {
      tokens.extend(Error::new_spanned(self, "Builder can only be derived for structs").to_compile_error());
      return;
    }

    let builder_ident = Ident::new(&format!("{ident}Builder"), ident.span());
    let struct_fields = data.as_ref().take_struct().unwrap();

    if !struct_fields.style.is_struct() {
      tokens.extend(Error::new_spanned(self, "Builder requires named struct fields").to_compile_error());
      return;
    }

    for field in struct_fields {
      let Field {
        ident, ty, attrs, ..
      } = field;

      if ident.is_none() {
        tokens.extend(Error::new_spanned(ident, "Field has no identifier").to_compile_error());
        return;
      }

      let ident = ident.as_ref().unwrap();
      let (ty, is_option) = get_option_inner_type(ty);

      init.push(quote! (#ident: std::option::Option::None));

      fields.push(quote! {
        #(#attrs)*
        #ident: std::option::Option<#ty>
      });

      let method = if field.into {
        quote! {
          pub fn #ident(mut self, #ident: impl Into<#ty>) -> Self {
            self.#ident = std::option::Option::Some(#ident.into());
            self
          }
        }
      } else {
        quote! {
          pub fn #ident(mut self, #ident: #ty) -> Self {
            self.#ident = std::option::Option::Some(#ident);
            self
          }
        }
      };

      methods.push(method);

      if is_option {
        build.push(quote! (#ident: self.#ident));
      } else if field.must {
        let msg = format!("Field '{}' must be initialized", ident);
        build.push(quote! (#ident: self.#ident.expect(#msg)));
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