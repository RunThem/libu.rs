use proc_macro::{TokenStream, TokenTree::Ident};
use proc_macro2::{Ident as Ident2, Span};
use quote::{ToTokens, quote};
use syn::{Expr, LocalInit, Stmt, parse, parse_quote};

pub fn clone(attr: TokenStream, item: TokenStream) -> TokenStream {
  // 过滤出标识符, 并生成克隆语句
  let idents = attr
    .into_iter()
    .filter(|tt| matches!(tt, Ident(_)))
    .map(|ident| {
      let ident = Ident2::new(&ident.to_string(), Span::call_site());
      Some(quote! { let #ident = #ident.clone(); })
    })
    .collect::<Vec<_>>();

  // 语句
  if let Ok(input) = parse::<Expr>(item.clone()) {
    quote! { { #(#idents)* #input }; }.into()
  }
  // 语句表达式
  else if let Ok(Stmt::Local(mut local)) = parse::<Stmt>(item.clone()) {
    let local_init = local.init.unwrap();
    let expr = local_init.expr;
    let block = parse_quote! { { #(#idents)* #expr } };

    local.init = Some(LocalInit {
      expr: Box::new(block),
      ..local_init
    });

    quote! { #local }.into()
  }
  // 其他不处理
  else {
    item
  }
}
