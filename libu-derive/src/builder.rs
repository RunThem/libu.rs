use darling::{ast, util};
use proc_macro2::{Ident, TokenStream as Ts};
use quote::quote;
use syn::{Attribute, Error, PathArguments, Type, TypePath, Visibility};

#[derive(Debug, darling::FromField)]
#[darling(attributes(builder), forward_attrs(allow, doc, cfg))]
pub(crate) struct Field {
  /// 字段标识符
  pub(crate) ident: Option<Ident>,
  /// 字段类型
  pub(crate) ty: Type,
  /// 字段属性, 只支持 allow/doc/cfg 三种属性
  pub(crate) attrs: Vec<Attribute>,

  /// 是否使入参为 Into<type>
  #[darling(default)]
  pub(crate) into: bool,

  /// 是否自动包装外壳
  // #[darling(default)]
  // pub(crate) wrap: bool,

  /// 是否必须初始化, 可以绕过没有实现 Default trait 的类型
  #[darling(default)]
  pub(crate) must: bool,
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(supports(struct_named), forward_attrs(allow, doc, cfg))]
pub(crate) struct BuilderDeriveInput {
  pub(crate) vis: Visibility,
  /// 结构体标识符
  pub(crate) ident: Ident,
  /// 结构体属性
  pub(crate) attrs: Vec<Attribute>,
  /// 结构体字段
  pub(crate) data: ast::Data<util::Ignored, Field>,
  // 泛型参数
  pub(crate) generics: ast::Generics<syn::GenericParam>,
}

impl quote::ToTokens for BuilderDeriveInput {
  fn to_tokens(&self, tokens: &mut Ts) {
    let mut Init = vec![];
    let mut Fields = vec![];
    let mut Method = vec![];
    let mut Build = vec![];

    let BuilderDeriveInput {
      vis,
      ident,
      attrs,
      data,
      generics,
    } = self;

    if !data.is_struct() {
      tokens.extend(Error::new_spanned(self, "Must define on Struct, not Enum").to_compile_error());
      return;
    }

    let builder_ident = Ident::new(&format!("{ident}Builder"), ident.span());
    let fields = data.as_ref().take_struct().unwrap();

    if !fields.style.is_struct() {
      tokens.extend(Error::new_spanned(self, "Struct style not is struct").to_compile_error());
      return;
    }

    // 遍历所有字段, 生成代码
    for field in fields {
      let Field {
        ident, ty, attrs, ..
      } = field;

      if ident.is_none() {
        tokens.extend(Error::new_spanned(ident, "Ident is None").to_compile_error());
        return;
      }

      let ident = ident.as_ref().unwrap();
      let (ty, is_option) = get_option_inner_type(ty);

      Init.push(quote! (#ident: std::option::Option::None));

      Fields.push(quote! {
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

      Method.push(method);

      if is_option {
        Build.push(quote! (#ident: self.#ident));
      } else if field.must {
        let msg = format!("Field '{ident}' is not initialized.");
        Build.push(quote! (#ident: self.#ident.expect(#msg)));
      } else {
        Build.push(quote! (#ident: self.#ident.unwrap_or_default()));
      }
    }

    let mut generics_meta_params = generics.params.clone();
    generics_meta_params.iter_mut().for_each(|t| {
      if let syn::GenericParam::Type(t) = t {
        t.bounds.clear();
        t.eq_token = None;
        t.default = None;
      }
    });

    let where_clause = &generics.where_clause;
    let generics_params = &generics.params;
    let generics_params = quote! (<#(#generics_params), *>);
    let generics_meta_params = quote! (<#(#generics_meta_params), *>);

    tokens.extend(quote! {
      #[derive(Default)]
      #(#attrs)*
      #vis struct #builder_ident #generics_params #where_clause {
        #(#Fields),*
      }

      impl #generics_params #builder_ident #generics_meta_params #where_clause {
        #(#Method)*

        pub fn build(mut self) -> #ident #generics_meta_params {
          #ident { #(#Build),* }
        }
      }

      impl #generics_params #ident #generics_meta_params #where_clause {
        pub fn builder() -> #builder_ident #generics_meta_params {
          #builder_ident { #(#Init),* }
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

fn build(mut ty: TypePath) -> Ts {
  let mut wraps = vec![];

  loop {
    let segment = &ty.path.segments.last().unwrap();
    if !["Rc", "Box", "RefCell"].iter().any(|x| segment.ident == x) {
      break;
    }

    wraps.push(syn::PathSegment::from(segment.ident.clone()));

    use syn::{AngleBracketedGenericArguments, GenericArgument, PathArguments, Type};
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
      &segment.arguments
    {
      if let GenericArgument::Type(Type::Path(s)) = &args[0] {
        ty = s.clone();
      }
    }
  }

  let mut ts = quote!(val);
  for t in wraps.iter().rev() {
    eprintln!("{:?}", t);
    ts = quote! (#t::new(#ts));
  }

  ts
}
