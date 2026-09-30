use crate::datex_proxy::data::StructureData;
use proc_macro2::TokenStream;
use quote::quote;

/// Generates the [DatexHash] implementations
pub fn generate_datex_hash(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[automatically_derived]
        impl #impl_generics DatexHash for #ident #ty_generics #where_clause {
            fn datex_hash(&self, hasher: &mut dyn core::hash::Hasher) {
                todo!()
            }
        }
    }
}
