use proc_macro2::TokenStream;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
  let input = match syn::parse2::<syn::DeriveInput>(input) {
    Ok(input) => input,
    Err(err) => return err.to_compile_error(),
  };
  let ident = input.ident;
  let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

  // SAFETY: The user has explicitly opted into this unsafe implementation.
  // They must ensure this type is actually safe to share across threads.
  quote::quote! {
    #[automatically_derived]
    unsafe impl #impl_generics ::core::marker::Sync for #ident #ty_generics #where_clause {}
  }
  .into()
}
