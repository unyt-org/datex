use proc_macro2::{TokenStream};
use quote::quote;
use syn::parse_quote;
use crate::datex_proxy::data::{Fields, Structure, StructureData};
use crate::datex_proxy::generator::helpers::{generate_struct_or_enum_variants_fields_mapping, SelfAccess, generate_from_parts_impl};

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

    let deserialize = generate_dif_deserialize_for_fields(
        &structure_data.structure,
    );

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        use serde::{
            Deserializer, Serializer,
            de::{DeserializeSeed, MapAccess, Visitor},
            ser::{SerializeMap, SerializeSeq, SerializeTuple},
        };

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

        impl<'de> DeserializeWithSerdeContext<'de> for #ident #ty_generics #where_clause {
            fn deserialize_with_ctx<D>(
                ctx: &SerdeContext<'_>,
                deserializer: D,
            ) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                #deserialize
            }
        }
    }
}

fn generate_dif_serialize_for_fields(
    fields: &Fields,
    tag: Option<&String>,
) -> TokenStream {
    match fields {
        Fields::Named(named_fields) => {
            let serialize_fields = named_fields.iter().map(|field| {
                let field_ident = &field.normalized_ident();
                quote! {
                    seq.serialize_element(&ValueWithSerdeContext::new(
                        #field_ident,
                        ctx,
                    ))?;
                }
            });
            let len = named_fields.len();
            quote! {
                let mut seq = serializer.serialize_seq(Some(#len))?;
                // #(#serialize_fields)*
                seq.end()
            }
        }
        Fields::Unnamed(unnamed_fields) => {
            let serialize_fields = unnamed_fields.iter().map(|field| {
                let field_ident = field.normalized_ident();
                quote! {
                    seq.serialize_element(&ValueWithSerdeContext::new(
                        #field_ident,
                        ctx,
                    ))?;
                }
            });
            let len = unnamed_fields.len();
            quote! {
                let mut seq = serializer.serialize_seq(Some(#len))?;
                // #(#serialize_fields)*
                seq.end()
            }
        }
        Fields::Unit => {
            quote! {
                todo!()
            }
        }
        Fields::Transparent(_) => {
            quote! {
                todo!()
            }
        }
    }
}

fn generate_dif_deserialize_for_fields(
    fields: &Structure,
) -> TokenStream {
    quote! {
        todo!()
    }
}