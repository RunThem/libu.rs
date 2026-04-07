use proc_macro::{TokenStream, TokenTree::Ident};
use proc_macro2::{Ident as Ident2, Span};
use quote::quote;
use syn::{Expr, LocalInit, Stmt, parse, parse_quote};

/// Auto-clone variables before an expression or closure.
///
/// Parses identifiers from the attribute and generates clone statements
/// that are inserted before the target expression.
pub fn clone(attr: TokenStream, item: TokenStream) -> TokenStream {
  // Filter identifiers and generate clone statements
  let clones = attr
    .into_iter()
    .filter(|tt| matches!(tt, Ident(_)))
    .map(|ident| {
      let ident = Ident2::new(&ident.to_string(), Span::call_site());
      quote! { let #ident = #ident.clone(); }
    })
    .collect::<Vec<_>>();

  // Handle expression
  if let Ok(expr) = parse::<Expr>(item.clone()) {
    quote! { { #(#clones)* #expr }; }.into()
  }
  // Handle let statement with closure
  else if let Ok(Stmt::Local(mut local)) = parse::<Stmt>(item.clone()) {
    let local_init = local.init.unwrap();
    let expr = local_init.expr;
    let block = parse_quote! { { #(#clones)* #expr } };

    local.init = Some(LocalInit {
      expr: Box::new(block),
      ..local_init
    });

    quote! { #local }.into()
  }
  // Return unchanged for other cases
  else {
    item
  }
}