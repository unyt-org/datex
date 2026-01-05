use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

pub fn create_async_test(input: ItemFn) -> TokenStream {
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;

    quote! {
        #[tokio::test]
        async fn #fn_name() {
            datex_core::run_async! {
                datex_core::native_global_context::
                #fn_body
            }
        }
    }
}
