use proc_macro2::TokenStream;
use quote::quote;

use crate::datex_proxy::data::{
    EnumVariant, Field, Fields, NamedField, SerdeMode, Structure,
    StructureData, TypeKind,
};

/// Generates the [GetDatexType] implementation for the given structure data.
/// Returns a TokenStream of the implementation.
pub fn generate_datex_proxy_types(
    structure_data: &StructureData,
) -> TokenStream {
    let datex_type = generate_type(structure_data);
    let StructureData {
        ident,
        generics,
        attributes,
        ..
    } = structure_data;
    let datex_name = structure_data
        .attributes
        .datex_name
        .clone()
        .unwrap_or(structure_data.ident.to_string());
    if matches!(attributes.type_kind, TypeKind::Structural { .. }) {
        quote! {
            #[automatically_derived]
            impl #generics GetDatexType for #ident #generics {
                fn datex_type(cache: &mut SharedReferencesCache) -> Type {
                    cache.resolve_structural_type::<Self, _>(
                        |cache| (#datex_type).with_name(#datex_name)
                    )
                }
            }
        }
    } else {
        quote! {
            #[automatically_derived]
            impl #generics GetDatexType for #ident #generics {
                fn datex_type(cache: &mut SharedReferencesCache) -> Type {
                    #datex_type
                }
            }
        }
    }
}

/// Generates a type definition for fields. Returns a TokenStream of [TypeDefinition].
fn generate_datex_type_definition(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Unit => quote! {
            TypeDefinition::UNIT
        },
        Fields::Named(fields) => {
            let field_types = fields
                .iter()
                .map(|f| named_field_to_definition(f))
                .collect::<Vec<_>>();
            quote! {
                TypeDefinition::Map(MapTypeDefinition(vec![
                    #(#field_types),*
                ]))
            }
        }
        Fields::Unnamed(field) => {
            let field_types = field
                .iter()
                .map(|f| field_to_definition(f))
                .collect::<Vec<_>>();

            quote! {
                TypeDefinition::List(ListTypeDefinition(vec![
                    #(#field_types),*
                ]))
            }
        }
        Fields::Transparent(field) => {
            let first_field = field_to_definition(field);
            quote! {
                #first_field.convert_to_definition()
            }
        }
    }
}

/// Generates a type definition for a single field. Returns a TokenStream of [TypeDefinition].
fn field_to_definition(field: &Field) -> TokenStream {
    let field_type = &field.ty;
    match &field.attributes.serde_mode {
        // no serde or infallible serde, provide/assume DatexValueContainerProxyInfallibleSerialize
        SerdeMode::None => {
            quote! {
                <#field_type as GetDatexType>::datex_type(cache.into())
            }
        }
        // Cannot infer type
        SerdeMode::Fallible | SerdeMode::Infallible => {
            quote! {
               Type::Definition(TypeDefinition::CoreType(CoreLibTypeId::Base(CoreLibBaseTypeId::Any)).into())
            }
        }
    }
}

/// Generates a type definition for a named field. Returns a TokenStream with a tuple of name and [TypeDefinition].
fn named_field_to_definition(field: &NamedField) -> TokenStream {
    let field_definition = field_to_definition(&field.field);
    let name = field.name.clone();
    quote! {
        (#name.to_string(), #field_definition)
    }
}

/// Generates a type definition for an enum. Returns a TokenStream of [TypeDefinition].
fn generate_datex_enum_type(enum_ty: &[EnumVariant]) -> TokenStream {
    let variants_datex_types = enum_ty.iter().map(|variant| {
        let name = &variant.name;
        let type_definition = generate_datex_type_definition(&variant.fields);
        quote! {
            Type::Definition(TypeDefinition::TaggedType(TaggedTypeDefinition {
                tag: #name.to_string(),
                ty: Some(Box::new(Type::Definition(#type_definition.into()))),
            }).into())
        }
    }); // FIXME do we need collect here?

    quote! {
        TypeDefinition::Union(UnionTypeDefinition(vec![
            #(#variants_datex_types),*
        ]))
    }
}

/// Wraps a type definition. Returns a TokenStream of [Type].
fn generate_type(structure_data: &StructureData) -> TokenStream {
    let type_definition = match &structure_data.structure {
        Structure::Enum(enum_val) => generate_datex_enum_type(enum_val),
        Structure::Struct(fields) => generate_datex_type_definition(fields),
    };
    if let TypeKind::Structural { .. } = structure_data.attributes.type_kind {
        quote! {
            Type::Definition(
                #type_definition.into()
            )
        }
    } else {
        // FIXME: calculate pointer address statically in macro at compile time
        let unique_name = format!(
            "{}::{}",
            structure_data.namespace.join("::"),
            structure_data.ident
        );
        let name = structure_data
            .attributes
            .datex_name
            .clone()
            .unwrap_or(structure_data.ident.to_string());
        quote! {{
            let address = unsafe {
                SelfOwnedPointerAddress::new_static_from_name(
                    #unique_name
                )
            };
            match unsafe {
                cache.reserve_shared_type(address.clone())
            } {
                SharedTypeReservation::Existing(ty) => {
                    Type::Entity(ty)
                }
                // if not found, create new def and register in cache
                SharedTypeReservation::New(ty) => {
                    cache.with_entity_boundary(|cache| {
                        let type_definition = #type_definition;

                        let impls = get_impls_for::<Self>(cache);

                        let definition = EntityTypeDefinition::new_with_impls(
                            type_definition.into(),
                            #name.to_string(),
                            impls,
                        );

                        cache.finish_shared_type(
                            address.clone(),
                            definition,
                        );

                        Type::Entity(ty)
                    })
                }
            }
        }}
    }
}
