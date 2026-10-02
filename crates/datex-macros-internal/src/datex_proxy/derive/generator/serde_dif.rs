use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse_quote;
use crate::datex_proxy::data::{Fields, StructureData};
use crate::datex_proxy::generator::helpers::{generate_struct_or_enum_variants_fields_mapping, SelfAccess};

/// Creates the implementation of the [SerializeSeed] and [DeserializeSeed] trait for the given structure data.
/// Returns a TokenStream of the implementation.
pub fn generate_serde_dif(
    structure_data: &StructureData,
) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let serialize = generate_struct_or_enum_variants_fields_mapping(
        &structure_data.structure,
        SelfAccess::BorrowedIdent {
            self_value: Ident::new("value", proc_macro2::Span::call_site()),
            self_type: parse_quote!(Self::Value),
        },
        generate_dif_serialize_for_fields,
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // FIXME: this doesnt work because of foreign trait for foreign struct impl
    quote! {
        impl<'ctx> SerializeSeed for #ident #ty_generics #where_clause {
            type Value = #ident #ty_generics;

            fn serialize_seed<S>(
                &mut self,
                ctx: &SerdeContext<'ctx>,
                serializer: S,
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