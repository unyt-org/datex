use crate::datex_proxy::data::{Fields, Structure, StructureData};
use proc_macro2::TokenStream;
use quote::quote;
use crate::datex_proxy::generator::helpers::{generate_struct_or_enum_variants_fields_mapping, map_enum_variants};

/// Generates the [FromParts] and [IntoParts] implementations
pub fn generate_convert_parts(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let into_parts_impl = generate_into_parts(structure_data);
    let from_parts_impl = generate_from_parts(structure_data);

    quote! {
        #into_parts_impl
        #from_parts_impl
    }
}


fn generate_into_parts(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let parts_kind = generate_struct_or_enum_variants_fields_mapping(
        &structure_data.structure,
        get_fields_parts_kind
    );

    quote! {
        #[automatically_derived]
        impl #generics IntoParts for #ident #generics {
            fn parts_kind(&self) -> PartsKind {
                #parts_kind
            }
        }
    }
}

/// Determines the [PartsKind] based on the fields of the structure
fn get_fields_parts_kind(fields: &Fields, tag: Option<&String>) -> TokenStream {
    match fields {
        Fields::Named(_) => quote! { PartsKind::Map },
        Fields::Unnamed(_) => quote! { PartsKind::List },
        Fields::Unit => quote! { PartsKind::None },
        Fields::Transparent(_) => quote! { _0.parts_kind() } // retrieve the parts kind of the single field
    }
}


fn generate_from_parts(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    quote! {
        #[automatically_derived]
        impl #generics FromParts for #ident #generics {}
    }
}