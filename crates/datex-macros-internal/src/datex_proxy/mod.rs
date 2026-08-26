mod derive;
mod impls;
mod module;
mod native_callable;

pub use derive::*;
pub use impls::*;
pub use module::*;

use proc_macro2::TokenStream;
use syn::Item;

pub fn generate_item_glue_code(
    args: Option<TokenStream>,
    input: TokenStream,
    item: Item,
) -> TokenStream {
    match &item {
        Item::Impl(item_impl) => {
            generate_impl_glue_code(args, input, item_impl)
        }
        Item::Mod(item_mod) => generate_mod_glue_code(args, item_mod),
        Item::Fn(item_fn) => {
            todo!("Implement glue code generation for functions");
        }
        e => {
            panic!(
                "The #[datex] attribute can not be applied to this item: {:?}.",
                e
            );
        }
    }
}
