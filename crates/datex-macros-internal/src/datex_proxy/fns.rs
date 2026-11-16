use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

pub fn generate_fn_glue_code(
    _args: Option<TokenStream>,
    _input: TokenStream,
    item: ItemFn,
) -> TokenStream {
    quote! {
        #item
    }
}
