use proc_macro2::TokenStream;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
  let input = match syn::parse2::<syn::DeriveInput>(input) {
    Ok(input) => input,
    Err(err) => return err.to_compile_error(),
  };
  let ident = input.ident;

  quote::quote! {
    /// SAFETY: The user has explicitly opted into this unsafe implementation.
    /// They must ensure this type is actually safe to share across threads.
    unsafe impl Sync for #ident {}
  }
  .into()
}
