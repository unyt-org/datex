use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse_quote;
use crate::datex_proxy::data::{Fields, StructureData};
use crate::datex_proxy::generator::helpers::{generate_struct_or_enum_variants_fields_mapping, SelfAccess};

/// Creates the implementation of the [SerializeWithSerdeContext] and [DeserializeWithSerdeContext] trait for the given structure data.
/// Returns a TokenStream of the implementation.
pub fn generate_serde_dif(
    structure_data: &StructureData,
) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let serialize = generate_struct_or_enum_variants_fields_mapping(
        &structure_data.structure,
        SelfAccess::Borrowed,
        generate_dif_serialize_for_fields,
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl<'ctx> SerializeWithSerdeContext for #ident #ty_generics #where_clause {
            fn serialize_with_ctx<S>(
                &self,
                ctx: &SerdeContext<'_>,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                #serialize
            }
        }
    }
}

fn generate_dif_serialize_for_fields(
    fields: &Fields,
    tag: Option<&String>,
) -> TokenStream {
    quote! {
        todo!()
    }
}