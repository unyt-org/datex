use crate::datex_proxy::data::StructureData;
use proc_macro2::TokenStream;
use quote::quote;

/// Generates the [Classification] and [StaticClassification] implementations
pub fn generate_classification(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics Classification for #ident #ty_generics #where_clause {}

        #[automatically_derived]
        impl #impl_generics StaticClassification for #ident #ty_generics #where_clause {}
    }
}
