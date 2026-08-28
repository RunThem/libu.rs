use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{Error, Expr, LocalInit, Stmt, parse_quote, parse2};

/// Auto-clone variables before an expression or closure.
///
/// Parses identifiers from the attribute and generates clone statements
/// that are inserted before the target expression.
///
/// The attribute must list one or more identifiers separated by commas. Any
/// other token (a path like `a.b`, a literal, …) or a duplicate identifier is
/// rejected, as is an empty attribute.
///
/// The macro applies to an expression or a `let` binding with an initializer:
///
/// ```text
/// #[clone(data)]
/// let handle = thread::spawn(move || { /* move a clone of `data` in */ });
/// ```
///
/// A `let` binding without an initializer, a non-`let` statement, or an item
/// (a `fn`, a `struct`, …) is rejected with a compile error instead of being
/// silently left alone.
pub fn clone(attr: TokenStream, item: TokenStream) -> TokenStream {
  match build(attr.into(), item.into()) {
    Ok(out) => out.into(),
    Err(err) => err.to_compile_error().into(),
  }
}

/// Shared implementation: returns the expanded tokens, or a `syn` error whose
/// span points at the offending attribute token / statement.
fn build(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2, Error> {
  let mut clones = Vec::new();
  let mut seen = Vec::new();

  for tt in attr {
    match tt {
      TokenTree::Ident(ident) => {
        let name = ident.to_string();
        if seen.contains(&name) {
          return Err(Error::new_spanned(
            &ident,
            format!("duplicate identifier `{name}` in `#[clone(...)]`"),
          ));
        }
        seen.push(name);
        clones.push(quote! { let #ident = #ident.clone(); });
      }
      // The separator comma is the only tolerated non-identifier token.
      TokenTree::Punct(punct) if punct.as_char() == ',' => {}
      other => {
        return Err(Error::new_spanned(
          &other,
          "expected an identifier, e.g. `#[clone(a, b)]`",
        ));
      }
    }
  }

  if clones.is_empty() {
    return Err(Error::new(
      Span::call_site(),
      "`#[clone(...)]` expects at least one identifier, e.g. `#[clone(a, b)]`",
    ));
  }

  // Expression position: `#[clone(x)] EXPR`.
  if let Ok(expr) = parse2::<Expr>(item.clone()) {
    return Ok(quote! { { #(#clones)* #expr } });
  }

  // Statement position: `#[clone(x)] let pat = <initializer>;`.
  if let Ok(stmt) = parse2::<Stmt>(item.clone()) {
    match stmt {
      Stmt::Local(mut local) => {
        let local_init = match local.init {
          Some(init) => init,
          // `let y;` has no initializer — report it instead of panicking on
          // `unwrap()`.
          None => {
            return Err(Error::new_spanned(
              &local,
              "`#[clone]` requires the `let` binding to have an initializer, e.g. `#[clone(x)] let y = expr;`",
            ));
          }
        };
        let expr = local_init.expr;
        let block = parse_quote! { { #(#clones)* #expr } };
        local.init = Some(LocalInit {
          expr: Box::new(block),
          ..local_init
        });
        Ok(quote! { #local })
      }
      Stmt::Expr(expr, semi) => {
        // `#[clone(x)] foo();` — an expression statement. Keep the trailing
        // semicolon when the statement had one.
        let block: Expr = parse_quote! { { #(#clones)* #expr } };
        if semi.is_some() {
          Ok(quote! { #block; })
        } else {
          Ok(quote! { #block })
        }
      }
      // An item (`fn`, `struct`, …) or a macro statement: not a target.
      _ => Err(Error::new_spanned(
        &stmt,
        "`#[clone]` can only be applied to an expression or a `let` statement with an initializer",
      )),
    }
  } else {
    // Not even a parseable statement or expression.
    Err(Error::new_spanned(
      &item,
      "`#[clone]` can only be applied to an expression or a `let` statement with an initializer",
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Expand `#[clone(attr)] item` and return the generated tokens.
  fn expand(attr: &str, item: &str) -> String {
    let attr = attr.parse().unwrap();
    let item = item.parse().unwrap();
    build(attr, item).unwrap().to_string()
  }

  fn expect_err(attr: &str, item: &str) {
    let attr_ts: TokenStream2 = attr.parse().unwrap();
    let item_ts: TokenStream2 = item.parse().unwrap();
    assert!(
      build(attr_ts, item_ts).is_err(),
      "expected `#[clone({attr})]` on `{item}` to be rejected"
    );
  }

  #[test]
  fn emits_clones_before_expression() {
    let out = expand("a, b", "f()");
    assert!(out.contains("let a = a . clone () ;"), "{out}");
    assert!(out.contains("let b = b . clone () ;"), "{out}");
    assert!(out.contains("f ()"), "{out}");
  }

  #[test]
  fn wraps_let_initializer() {
    assert!(
      expand("x", "let y = f();").starts_with("let y = {"),
      "the `let` initializer must be wrapped in a block, not replaced"
    );
  }

  #[test]
  fn rejects_non_identifiers() {
    expect_err("a.b", "f()"); // a path is not a single identifier
  }

  #[test]
  fn rejects_duplicates() {
    expect_err("a, a", "f()");
  }

  #[test]
  fn rejects_empty_attribute() {
    expect_err("", "f()");
  }

  #[test]
  fn rejects_let_without_initializer() {
    expect_err("x", "let y;");
  }

  #[test]
  fn rejects_items() {
    expect_err("x", "fn foo() {}");
  }
}
