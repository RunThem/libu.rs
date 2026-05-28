use proc_macro::TokenStream;
use quote::quote;
use syn::{
  Expr, Token,
  parse::{Parse, ParseStream},
};

struct SelectInput(Vec<(Expr, Expr)>);

impl Parse for SelectInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut arms = Vec::new();

    while !input.is_empty() {
      let receiver: Expr = input.parse()?;

      input.parse::<Token![=>]>()?;

      let handler: Expr = input.parse()?;

      input.parse::<Token![,]>();

      arms.push((receiver, handler));
    }

    Ok(SelectInput(arms))
  }
}

pub fn select(input: TokenStream) -> TokenStream {
  let selects = match syn::parse::<SelectInput>(input.into()) {
    Ok(input) => input,
    Err(e) => return e.to_compile_error().into(),
  };

  let calls = selects.0.iter().map(|arm| {
    let receiver = &arm.0;
    let handler = &arm.1;

    quote! { .recv(#receiver, #handler) }
  });

  quote! { ::flume::Selector::new() #(#calls)* .wait() }.into()
}
