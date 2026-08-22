use crate::utils::get_datex_core_crate_name;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Item, ItemEnum, ItemMod, ItemStruct, Meta};
pub fn generate_mod_glue_code(
    input: TokenStream,
    item: &ItemMod,
) -> TokenStream {
    let Some((_, items)) = &item.content else {
        return quote! {
            #input
        };
    };

    let module_name = item.ident.to_string();

    let attrs = strip_datex_attributes(&item.attrs);
    let vis = &item.vis;
    let ident = &item.ident;

    quote! {
        #(#attrs)*
        #vis mod #ident {
            #(#items)*
        }
    }
}
fn strip_datex_attributes(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("datex"))
        .cloned()
        .collect()
}
