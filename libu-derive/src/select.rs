use proc_macro::TokenStream;
use quote::quote;
use syn::{
  Expr, Token,
  parse::discouraged::Speculative,
  parse::{Parse, ParseStream},
};

struct SelectInput {
  crate_path: Option<syn::Path>,
  arms: Vec<(Expr, Expr)>,
}

impl Parse for SelectInput {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Optional crate path prefix: `select!(::my_flume; &rx => h, ...)`.
    // Detected by forking: a leading path followed by `;` is the crate
    // path, anything else starts an arm (a bare `rx` is a path too).
    let mut crate_path = None;
    let fork = input.fork();
    if let Ok(path) = fork.parse::<syn::Path>() {
      if fork.peek(Token![;]) {
        crate_path = Some(path);
        input.advance_to(&fork);
        input.parse::<Token![;]>()?;
      }
    }

    let mut arms = Vec::new();

    while !input.is_empty() {
      let receiver: Expr = input.parse()?;

      input.parse::<Token![=>]>()?;

      let handler: Expr = input.parse()?;

      // Commas are required between arms; a missing one would otherwise be
      // swallowed into the handler expression (e.g. `h &rx2` parses as a
      // bitand).
      if !input.is_empty() {
        input.parse::<Token![,]>()?;
      }

      arms.push((receiver, handler));
    }

    Ok(SelectInput { crate_path, arms })
  }
}

pub fn select(input: TokenStream) -> TokenStream {
  select2(input.into()).into()
}

fn select2(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
  let selects = match syn::parse2::<SelectInput>(input) {
    Ok(input) => input,
    Err(e) => return e.to_compile_error(),
  };

  if selects.arms.is_empty() {
    // An empty Selector panics at runtime (modulo-zero in flume's wait),
    // so reject it at compile time.
    return quote! {
      compile_error!("select! requires at least one arm")
    };
  }

  let flume_path = match &selects.crate_path {
    Some(path) => quote! (#path),
    None => quote!(::flume),
  };

  let calls = selects.arms.iter().map(|arm| {
    let receiver = &arm.0;
    let handler = &arm.1;

    // flume's Selector::recv takes `&Receiver`; accept either `rx` or `&rx`.
    let receiver = if matches!(receiver, Expr::Reference(_)) {
      quote! (#receiver)
    } else {
      quote! (&(#receiver))
    };

    quote! { .recv(#receiver, #handler) }
  });

  quote! { #flume_path::Selector::new() #(#calls)* .wait() }
}

#[cfg(test)]
mod tests {
  use super::select2;

  fn expand(input: &str) -> String {
    let ts: proc_macro2::TokenStream = input.parse().unwrap();
    select2(ts).to_string()
  }

  #[test]
  fn auto_references_bare_receivers() {
    let out = expand("rx1 => |m| m, &rx2 => |m| m");

    // Bare receiver gets a leading &; an explicit & is kept as-is.
    assert!(out.contains("recv (& (rx1) ,"), "{out}");
    assert!(out.contains("recv (& rx2 ,"), "{out}");
  }

  #[test]
  fn empty_input_rejected() {
    let out = expand("");

    assert!(out.contains("requires at least one arm"), "{out}");
  }

  #[test]
  fn missing_comma_rejected() {
    let out = expand("rx1 => |m| m rx2 => |m| m");

    assert!(out.contains("expected `,`"), "{out}");
  }

  #[test]
  fn custom_crate_path() {
    let out = expand("::my_flume; rx1 => |m| m");
    assert!(out.contains(":: my_flume :: Selector :: new ()"), "{out}");

    let out2 = expand("my::flume; rx1 => |m| m");
    assert!(out2.contains("my :: flume :: Selector :: new ()"), "{out2}");
  }

  #[test]
  fn default_crate_path_is_flume() {
    let out = expand("rx1 => |m| m");

    assert!(out.contains(":: flume :: Selector :: new ()"), "{out}");
  }

  #[test]
  fn expansion_shape() {
    let out = expand("rx1 => |m| m");

    assert!(out.contains(":: flume :: Selector :: new ()"), "{out}");
    assert!(out.contains(". recv (& (rx1) , | m | m)"), "{out}");
    assert!(out.contains(". wait ()"), "{out}");
  }
}
