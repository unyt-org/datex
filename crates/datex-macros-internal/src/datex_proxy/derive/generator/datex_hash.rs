use proc_macro2::TokenStream;
use quote::quote;
use crate::datex_proxy::data::StructureData;

/// Generates the [DatexHash] implementations
pub fn generate_datex_hash(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    quote! {
        #[automatically_derived]
        impl #generics DatexHash for #ident #generics {
            fn datex_hash(&self) -> u64 {
                todo!()
            }
        }
    }
}