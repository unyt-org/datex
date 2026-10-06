use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse_quote;
use crate::datex_proxy::data::{AnyField, FieldIdent, FieldType, Fields, IndexedField, NamedField, Structure, StructureData};
use crate::datex_proxy::generator::helpers::{generate_struct_or_enum_variants_fields_mapping, SelfAccess, generate_from_parts_impl, generate_struct_field_accessors, map_enum_variants};

/// Creates the implementation of the [SerializeWithSerdeContext] and [DeserializeWithSerdeContext] trait for the given structure data.
/// Returns a TokenStream of the implementation.
pub fn generate_serde_dif(
    structure_data: &StructureData,
) -> TokenStream {

    let serialize_impl = generate_serde_serialize(structure_data);

    let deserialize_impl = if !structure_data.attributes.no_deserialize {
        generate_serde_deserialize(structure_data)
    } else {
        generate_unimplemented_serde_deserialize(structure_data)
    };

    quote! {
        use serde::{
            Deserializer, Serializer,
            de::{DeserializeSeed, MapAccess, SeqAccess, Error, Visitor},
            ser::{SerializeMap, SerializeSeq, SerializeTuple},
        };

        #serialize_impl
        #deserialize_impl
    }
}

fn generate_serde_serialize(
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


fn generate_serde_deserialize(
    structure_data: &StructureData,
) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let (deserialize_impl, visitor_impl) = match &structure_data.structure {
        Structure::Struct(fields) => {
            generate_dif_deserialize_for_struct(
                structure_data,
                fields,
            )
        }
        Structure::Enum(variants) => {
            // TODO
            (quote! { todo!() }, quote! { })
        }
    };

    generate_serde_deserialize_skeleton(structure_data, deserialize_impl, visitor_impl)
}

/// Generates a deserialization implementation that returns an error indicating that deserialization is not implemented for the given structure.
fn generate_unimplemented_serde_deserialize(
    struct_data: &StructureData,
) -> TokenStream {
    let deserialize_impl = quote! {
        Err(D::Error::custom("Deserialization is not implemented for this type"))
    };

    generate_serde_deserialize_skeleton(struct_data, deserialize_impl, quote! { })
}

/// Generates the skeleton of the deserialization implementation and Visitor for a struct
fn generate_serde_deserialize_skeleton(
    structure_data: &StructureData,
    deserialize_impl: TokenStream,
    visitor_impl: TokenStream,
) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for #ident #ty_generics #where_clause {
            fn deserialize_with_ctx<D>(
                ctx: &SerdeContext<'_>,
                deserializer: D,
            ) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                #deserialize_impl
            }
        }

        #visitor_impl
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
                    map.serialize_entry(stringify!(#field_ident), &ValueWithSerdeContext::new(
                        #field_ident,
                        ctx,
                    ))?;
                }
            });
            let len = named_fields.len();
            quote! {
                let mut map = serializer.serialize_map(Some(#len))?;
                #(#serialize_fields)*
                map.end()
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
                #(#serialize_fields)*
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

/// Generates the deserialization implementation and Visitor for a struct.
fn generate_dif_deserialize_for_struct(
    structure_data: &StructureData,
    fields: &Fields,
) -> (TokenStream, TokenStream) {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let collector_ident = Ident::new(&format!("{}DeserializeCollector", ident), ident.span());

    let (properties, property_defaults, property_collapses) = generate_property_mappings(fields);

    let collapse_init = fields.wrap_in_initializer(property_collapses.into_iter());

    let deserialize_impl = quote! {
        deserializer.deserialize_any(#collector_ident::new(ctx))
    };

    let visitor_methods = match fields {
        Fields::Named(named) => {
            generate_visitor_methods_for_named_fields(named)
        }
        Fields::Unnamed(unnamed) => {
            generate_visitor_methods_for_unnamed_fields(unnamed)
        }
        Fields::Transparent(_) => {
            // TODO:
            quote! {

            }
        }
        Fields::Unit => {
            // TODO:
            quote! {

            }
        }
    };

    let visitor_impl = quote! {

        struct #collector_ident<'a, 'ctx> {
            pub __ctx: &'a SerdeContext<'ctx>,
            #(#properties),*
        }

        impl<'a, 'ctx> #collector_ident<'a, 'ctx> {
            fn new(ctx: &'a SerdeContext<'ctx>) -> Self {
                #collector_ident::from(ctx)
            }
            fn collapse(self) -> Result<#ident, &'static str> {
                Ok(#ident #collapse_init)
            }
        }

        impl<'a, 'ctx> From<&'a SerdeContext<'ctx>> for #collector_ident<'a, 'ctx> {
            fn from(ctx: &'a SerdeContext<'ctx>) -> Self {
                #collector_ident {
                    __ctx: ctx,
                    #(#property_defaults),*
                }
            }
        }

        impl<'a, 'ctx, T> From<&'a DeserializeSerdeContext<'a, 'ctx, T>> for #collector_ident<'a, 'ctx> {
            fn from(value: &'a DeserializeSerdeContext<'a, 'ctx, T>) -> Self {
                #collector_ident::from(value.ctx)
            }
        }

        impl<'a, 'ctx, T> From<&'a #collector_ident<'a, 'ctx>> for DeserializeSerdeContext<'a, 'ctx, T> {
            fn from(value: &'a #collector_ident<'a, 'ctx>) -> Self {
                DeserializeSerdeContext::new(value.__ctx)
            }
        }

        impl<'de, 'a, 'ctx> Visitor<'de> for #collector_ident<'a, 'ctx> {
            type Value = #ident;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(
                    "either an object with string keys or a sequence of [key, value] entries",
                )
            }

            #visitor_methods
        }
    };

    (deserialize_impl, visitor_impl)
}

fn generate_visitor_methods_for_named_fields(
    fields: &[NamedField]
) -> TokenStream {

    let map_key_deserializers = fields.iter().map(|field| {
        let field_name = field.datex_field_name();
        let normalized_ident = field.normalized_ident();
        let ty = field.ty();
        quote! {
            #field_name => {
                let value: #ty = map.next_value_seed(DeserializeSerdeContext::<#ty>::new(self.__ctx))?;
                self.#normalized_ident = Some(value);
            }
        }
    });


    quote! {
        fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            while let Some(key) = map.next_key::<String>()? {
                match key {
                    #(#map_key_deserializers),*
                    _ => {
                        // Ignore unknown keys
                        let _: serde::de::IgnoredAny = map.next_value()?;
                    }
                }
            }

            self.collapse().map_err(|err| A::Error::missing_field(err))
        }

        fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            self.collapse().map_err(|err| A::Error::missing_field(err))
        }
    }
}

fn generate_visitor_methods_for_unnamed_fields(
    fields: &[IndexedField]
) -> TokenStream {
    quote! {

    }
}


fn generate_property_mappings(
    fields: &Fields,
) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {

    fields.iter().map(|field| {
        let field_ty = field.ty();
        let normalized_ident = field.normalized_ident();
        let field_name = field.original_name();

        let field_init = quote! {
            pub #normalized_ident: Option<#field_ty>
        };

        let field_default = quote! {
            #normalized_ident: None
        };

        let field_collapse = quote! {
            self.#normalized_ident.ok_or_else(|| #field_name)?
        };

        (field_init, field_default, field_collapse)
    }).collect()
}