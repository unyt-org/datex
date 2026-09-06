use crate::datex_proxy::data::StructureData;
use proc_macro2::TokenStream;
use quote::quote;

/// Generates the [Classification] and [StaticClassification] implementations
pub fn generate_classification(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    quote! {
        #[automatically_derived]
        impl #generics Classification for #ident #generics {}

        #[automatically_derived]
        impl #generics StaticClassification for #ident #generics {}
    }
}
