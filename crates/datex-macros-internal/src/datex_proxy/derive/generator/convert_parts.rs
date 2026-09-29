use crate::datex_proxy::data::{Field, FieldMapping, Fields, Structure, StructureData};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use crate::datex_proxy::generator::helpers::{generate_struct_or_enum_variants_fields_mapping, map_enum_variants, SelfAccess};

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
        SelfAccess::Borrowed,
        get_fields_parts_kind
    );

    let into_map_parts_impl = generate_struct_or_enum_variants_fields_mapping(
        &structure_data.structure,
        SelfAccess::Moved,
        generate_into_map_parts_for_fields
    );
    
    let into_list_parts_impl = generate_struct_or_enum_variants_fields_mapping(
        &structure_data.structure,
        SelfAccess::Moved,
        generate_into_list_parts_for_fields
    );

    quote! {
        #[automatically_derived]
        impl #generics IntoParts for #ident #generics {
            fn parts_kind(&self) -> PartsKind {
                #parts_kind
            }

            fn try_into_map_parts<'a>(
                self: Box<Self>,
                _cache: &'a mut SharedReferencesCache,
            ) -> Result<Map, ()>
            where
                Self: 'a,
            {
                #into_map_parts_impl
            }
            
            fn try_into_list_parts<'a>(
                self: Box<Self>,
                _cache: &'a mut SharedReferencesCache
            ) -> Result<List, ()>
            where
                Self: 'a,
            {
                #into_list_parts_impl
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
        Fields::Transparent(field) => {
            let accessor = field.normalized_ident();
            quote! { #accessor.parts_kind() } // retrieve the parts kind of the single field
        }
    }
}

fn generate_into_map_parts_for_fields(fields: &Fields, tag: Option<&String>) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let yields = fields.iter().map(|field| {
                let field_name = &field.name;
                let field_value_container = value_container_from_field(
                    field.normalized_ident(),
                    &field.field.attributes.field_mapping,
                );
                quote! {
                    yield (
                        ValueContainer::from(#field_name.to_string()),
                        #field_value_container,
                    );
                }
            });
            quote! {
                // gen block that yields (ValueContainer, ValueContainer) pairs for each field in the struct
                let map = gen move {
                    #(#yields)*
                };
                Ok(map.collect::<Map>())
            }
        },
        Fields::Unit | Fields::Unnamed(_) => quote! { Err(()) }, // unit structs cannot be converted into parts
        Fields::Transparent(field) => {
            let accessor = field.normalized_ident();
            quote! { Box::new(#accessor).try_into_map_parts(_cache) } // delegate to the single field's implementation
        }
    }
}


fn generate_into_list_parts_for_fields(fields: &Fields, tag: Option<&String>) -> TokenStream {
    match fields {
        Fields::Unnamed(fields) => {
            let yields = fields.iter().map(|field| {
                let field_value_container = value_container_from_field(
                    field.normalized_ident(),
                    &field.field.attributes.field_mapping,
                );
                quote! {
                    yield #field_value_container;
                }
            });
            quote! {
                // gen block that yields ValueContainer for each field in the tuple struct
                let list = gen move {
                    #(#yields)*
                };
                Ok(list.collect::<List>())
            }
        },
        Fields::Unit | Fields::Named(_) => quote! { Err(()) }, // unit structs cannot be converted into parts
        Fields::Transparent(field) => {
            let accessor = field.normalized_ident();
            quote! { Box::new(#accessor).try_into_list_parts(_cache) } // delegate to the single field's implementation
        }
    }
}


/// Generates the appropriate ValueContainer conversion based on the field's mapping
/// For fields with serde mapping, it uses `serde_to_value_container`, otherwise it uses `ValueContainer::from`.
fn value_container_from_field(
    accessor: Ident,
    field_mapping: &FieldMapping,
) -> TokenStream {
    if field_mapping.is_serde() {
        quote! { serde_to_value_container(#accessor) }
    } else {
        quote! { ValueContainer::from(#accessor) }
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