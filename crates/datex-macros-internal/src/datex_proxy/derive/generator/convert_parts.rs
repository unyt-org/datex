use crate::datex_proxy::data::StructureData;
use proc_macro2::TokenStream;
use quote::quote;

/// Generates the [FromParts] and [IntoParts] implementations
pub fn generate_convert_parts(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    quote! {
        #[automatically_derived]
        impl #generics FromParts for #ident #generics {}

        #[automatically_derived]
        impl #generics IntoParts for #ident #generics {}
    }
}
