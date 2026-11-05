mod derive;
mod fns;
pub mod helpers;
mod impls;
mod module;
mod native_callable;

pub use derive::*;
pub use impls::*;
pub use module::*;

use crate::datex_proxy::fns::generate_fn_glue_code;
use proc_macro2::TokenStream;
use syn::Item;

pub fn generate_item_glue_code(
    args: Option<TokenStream>,
    input: TokenStream,
    item: Item,
) -> TokenStream {
    match item {
        Item::Impl(item_impl) => {
            generate_impl_glue_code(args, input, item_impl)
        }
        Item::Mod(item_mod) => generate_mod_glue_code(args, item_mod),
        Item::Fn(item_fn) => generate_fn_glue_code(args, input, item_fn),
        e => {
            panic!(
                "The #[datex] attribute can not be applied to this item: {:?}.",
                e
            );
        }
    }
}
