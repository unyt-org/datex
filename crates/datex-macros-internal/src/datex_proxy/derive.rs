use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Meta, Token,
    punctuated::Punctuated,
};

use crate::utils::get_project_relative_file_path;

#[derive(Debug, PartialEq)]
enum SerdeMode {
    /// Serde serializable/deserializable fields are not allowed inside the datex proxy value.
    /// Since the generated code will not attempt to serialize any fields with serde,
    /// it will only provide an infallible into method to convert to ValueContainer
    None,
    /// Serde serializable/deserializable fields are allowed inside the datex proxy value.
    /// It is assumed that the serialization might fail, so the generated code will only provide a
    /// try_into method to convert to ValueContainer
    Fallible,
    /// Serde serializable/deserializable fields are allowed inside the datex proxy value.
    /// The user explicitly guarantees that the serialization will not fail, so the generated code will
    /// provide an infallible into method to convert to ValueContainer
    Infallible,
}

#[derive(Debug, PartialEq)]
enum FieldsType {
    Named,
    Unnamed,
    Unit,
    Transparent,
}

impl From<&Fields> for FieldsType {
    fn from(fields: &Fields) -> Self {
        match &fields {
            Fields::Unit => FieldsType::Unit,
            Fields::Named(_) => FieldsType::Named,
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    FieldsType::Transparent
                } else {
                    FieldsType::Unnamed
                }
            }
        }
    }
}

/// Per-field attributes for the Datex derive macro
#[derive(Debug, PartialEq)]
pub struct FieldAttributes {
    serde_mode: SerdeMode,
    datex_rename: Option<String>,
    datex_default: bool,
    datex_skip: bool,
}

/// Top-level attributes for the Datex derive macro
#[derive(Debug, PartialEq)]
pub struct TopLevelAttributes {
    /// Internally used attribute to indicate that the macro should use the `datex_core` namespace
    /// instead of inferring it. This is required for doctests to work.
    force_datex_core_namespace: bool,

    /// Optional override for the exported nameme of the type. Defaults to the Rust struct or enum name.
    datex_name: Option<String>,

    /// If the decorated struct or enum should be exported to the Datex registry.
    /// `#[datex(export)]`
    export: bool,
    namespace: Option<String>,
    docs: Option<String>,
}

pub struct DeriveData {
    is_fallible_serialization: bool,
    from_datex_fields_inner: TokenStream,
    into_datex_fields_inner: TokenStream,
    datex_type: TokenStream,
    helpers: Option<TokenStream>,
}

/// Derive implementation for the [Datex] derive macro.
pub fn derive(input: DeriveInput) -> TokenStream {
    let top_level_attributes = parse_top_level_attributes(&input.attrs);

    let datex_core_crate_name =
        if top_level_attributes.force_datex_core_namespace {
            Ident::new("datex_core", Span::call_site())
        } else {
            get_datex_core_crate_name()
        };

    let DeriveData {
        into_datex_fields_inner,
        from_datex_fields_inner,
        datex_type,
        is_fallible_serialization,
        helpers,
    } = match input.data {
        Data::Struct(data_struct) => derive_struct(data_struct, &input.ident),
        Data::Enum(data_enum) => derive_enum(data_enum, &input.ident),
        _ => unimplemented!(),
    };

    let ident = input.ident;
    let datex_name = top_level_attributes
        .datex_name
        .clone()
        .unwrap_or_else(|| ident.to_string());

    let docs = match &top_level_attributes.docs {
        Some(docs) => quote! {
            Some(#docs)
        },
        None => quote! {
            None
        },
    };

    let export = true; // FIXME shall we opt-in or opt-out from top_level_attributes.export;
    let namespace = &top_level_attributes.namespace.unwrap_or_else(|| {
        let mut ns = get_project_relative_file_path();
        ns.set_extension("");
        ns.to_str()
            .expect("Failed to convert file path to string")
            .to_string()
    });

    let registration = if export {
        quote! {
            #datex_core_crate_name::inventory::submit! {
                #datex_core_crate_name::datex_registry::DatexRegistration::new::<#ident>(
                    #datex_core_crate_name::datex_registry::DatexMetadata {
                        name: #datex_name,
                        rust_ident: stringify!(#ident),
                        docs: #docs,
                        export: #export,
                        namespace: #namespace,
                    }
                )
            }
        }
    } else {
        quote! {}
    };

    let serialize = match is_fallible_serialization {
        // no serde or infallible serde, provide/assume DatexValueContainerProxyInfallibleSerialize
        false => {
            quote! {
                #[automatically_derived]
                impl From<#ident> for Value {
                    fn from(value: #ident) -> Self {
                        #into_datex_fields_inner
                    }
                }

                #[automatically_derived]
                impl DatexValueProxySerialize for #ident {
                    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
                        Ok(self.into())
                    }
                }

                #[automatically_derived]
                impl DatexValueProxyInfallibleSerialize for #ident {
                    fn to_value(self) -> Value {
                       self.into()
                    }
                }
            }
        }
        true => {
            quote! {
                #[automatically_derived]
                impl TryFrom<#ident> for Value {
                    type Error = TryToDatexValueError;

                    fn try_from(value: #ident) -> Result<Self, Self::Error> {
                        Ok(#into_datex_fields_inner)
                    }
                }

                #[automatically_derived]
                impl TryFrom<#ident> for ValueContainer {
                    type Error = TryToDatexValueError;

                    fn try_from(value: #ident) -> Result<Self, Self::Error> {
                        Ok(ValueContainer::Local(Value::from(#into_datex_fields_inner)))
                    }
                }

                #[automatically_derived]
                impl DatexValueProxySerialize for #ident {
                    fn try_to_value(self) -> Result<Value, TryToDatexValueError> {
                        self.try_into()
                    }
                }
            }
        }
    };

    quote! {
        const _: () = {
            use #datex_core_crate_name::{
                datex_proxy::{
                    DatexValueContainerProxy,
                    DatexValueContainerProxyInfallibleSerialize,
                    DatexValueContainerProxySerialize,
                    DatexValueContainerProxyDeserialize,
                    DatexValueProxy,
                    DatexValueProxyInfallibleSerialize,
                    DatexValueProxySerialize,
                    DatexValueProxyDeserialize,
                    TryToDatexValueError,
                    TryFromDatexValueError,
                    DatexProxyTypes,
                    serde_compat::{
                        try_serde_to_value_container,
                        try_serde_from_value_container
                    }
                },
                types::r#type::Type,
                types::literal_type_definition::LiteralTypeDefinition,
                runtime::memory::Memory,
                libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
                values::value_container::ValueContainer,
                values::value::Value,
                values::core_value::CoreValue,
                values::core_values::map::Map,
                values::core_values::list::List,
                types::type_definition::TypeDefinition,
                types::type_definition::union::UnionTypeDefinition,
                types::type_definition::map::MapTypeDefinition,
                types::type_definition::list::ListTypeDefinition,
                types::type_definition::tagged_type::TaggedTypeDefinition,
                prelude::*
            };

            #[automatically_derived]
            impl DatexValueProxy for #ident {}

            #helpers

            #serialize

            #[automatically_derived]
            impl DatexValueProxyDeserialize for #ident {
                fn try_from_value(
                    value: Value,
                ) -> Result<Self, TryFromDatexValueError> {
                   value.try_into()
                }
            }

            #[automatically_derived]
            impl TryFrom<Value> for #ident {
                type Error = TryFromDatexValueError;

                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    Ok(#from_datex_fields_inner)
                }
            }

            #[automatically_derived]
            impl TryFrom<ValueContainer> for #ident {
                type Error = TryFromDatexValueError;

                fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
                    match value {
                        ValueContainer::Local(value) => value.try_into(),
                        _ => Err(TryFromDatexValueError("Expected ValueContainer::Local".to_string())),
                    }
                }
            }

            #[automatically_derived]
            impl DatexProxyTypes for #ident {
                fn datex_type(memory: &mut Memory) -> Type {
                    (#datex_type).with_name(#datex_name)
                }
            }

            #registration
        };
    }
}

/// Derive implementation for structs
fn derive_struct(data_struct: DataStruct, ident: &Ident) -> DeriveData {
    let FieldDeriveData {
        is_fallible_serialization,
        fields_type,
        into_datex_fields,
        from_datex_fields,
        datex_type,
        field_names,
    } = derive_fields(&data_struct.fields);

    let into_datex_fields_inner = match fields_type {
        FieldsType::Named => quote! {
            Value::from(Map::StructuralWithStringKeys(vec![
                #(#into_datex_fields),*
            ]))
        },
        FieldsType::Unnamed => {
            quote! {
                Value::from(List::from(vec![
                    #(#into_datex_fields),*
                ]))
            }
        }
        FieldsType::Transparent => {
            let into_field = into_datex_fields.first().unwrap();
            quote! {
                let container = #into_field;
                if let ValueContainer::Local(value) = container {
                    value
                }
                else {
                    unreachable!("Expected ValueContainer::Local");
                }
            }
        }
        FieldsType::Unit => quote! {
            Value::null()
        },
    };

    let from_datex_fields_inner = match fields_type {
        FieldsType::Named => quote! {{
            let mut map: Map = value.try_into()?;
            map.ensure_only_properties(&[
                #(#field_names),*
            ])
            .map_err(|err| TryFromDatexValueError(err.to_string()))?;

            #ident {
                #(#from_datex_fields),*
            }
        }},
        FieldsType::Transparent => {
            let from_field = from_datex_fields.first().unwrap();
            quote! {{
                #ident(#from_field)
            }}
        }
        FieldsType::Unnamed => {
            quote! {{
                let mut list: List = value.try_into()?;

                #ident(
                    #(#from_datex_fields),*
                )
            }}
        }
        FieldsType::Unit => quote! {
            if !value.is_null() {
                return Err(TryFromDatexValueError(format!("Unexpected value, expected null")));
            }
            #ident
        },
    };

    let type_definition = datex_type.unwrap_or_else(|| {
        quote! {
            Type::Alias(TypeDefinition::CoreType(CoreLibBaseTypeId::Unit.into()))
        }
    });

    DeriveData {
        is_fallible_serialization,
        into_datex_fields_inner,
        from_datex_fields_inner,
        datex_type: type_definition,
        helpers: None,
    }
}

fn derive_enum(data_enum: DataEnum, ident: &Ident) -> DeriveData {
    // serialization is only infallible if the enum has no serde fields or only serde fields with datex(serde_infallible)
    let mut is_fallible_serialization = false;

    let mut variants_into_datex_fields = Vec::<TokenStream>::new();
    let mut variants_from_datex_fields = Vec::<TokenStream>::new();
    let mut variants_datex_types = Vec::<TokenStream>::new();

    let mut helper_structs = Vec::<TokenStream>::new();

    for variant in data_enum.variants {
        let variant_name = variant.ident.to_string();
        let variant_ident = &variant.ident;
        // create helper struct {variant_name}Inner
        let helper_struct_ident = format_ident!("{}Inner", variant_ident);

        helper_structs.push(generate_enum_helper_structs(
            &variant.fields,
            ident,
            variant_ident,
            &helper_struct_ident,
        ));

        let FieldDeriveData {
            is_fallible_serialization: variant_is_fallible_serialization,
            fields_type,
            into_datex_fields,
            from_datex_fields,
            datex_type,
            field_names,
        } = derive_fields(&variant.fields);

        // if any variant is fallible, mark as fallible
        if variant_is_fallible_serialization {
            is_fallible_serialization = true;
        }

        let into_datex_fields_inner = match fields_type {
            FieldsType::Named => quote! {
                #ident::#variant_ident {..} => {
                    let value: #helper_struct_ident = value.into();
                    let map = Map::StructuralWithStringKeys(vec![
                        #(#into_datex_fields),*
                    ]);
                    Value {
                        inner: CoreValue::Map(map),
                        custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
                            tag: #variant_name.to_string(),
                            ty: None,
                        })),
                    }
                }
            },
            FieldsType::Transparent => {
                let into_field = into_datex_fields.first().unwrap();
                quote! {
                    #ident::#variant_ident {..} => {
                        let value: #helper_struct_ident = value.into();
                        let container = #into_field;
                        if let ValueContainer::Local(Value {custom_type: None, inner}) = container {
                            Value {
                                inner,
                                custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
                                    tag: #variant_name.to_string(),
                                    ty: None,
                                })),
                            }
                        }
                        else {
                            unreachable!("Expected ValueContainer::Local without custom type");
                        }
                    }
                }
            }
            FieldsType::Unnamed => {
                quote! {
                    #ident::#variant_ident (..) => {
                        let value: #helper_struct_ident = value.into();
                        let list = List::from(vec![
                            #(#into_datex_fields),*
                        ]);
                        Value {
                            inner: CoreValue::List(list),
                            custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
                                tag: #variant_name.to_string(),
                                ty: None,
                            })),
                        }
                    }
                }
            }
            FieldsType::Unit => quote! {
                #ident::#variant_ident => {
                    Value {
                        inner: CoreValue::Null,
                        custom_type: Some(TypeDefinition::TaggedType(TaggedTypeDefinition {
                            tag: #variant_name.to_string(),
                            ty: None,
                        })),
                    }
                }
            },
        };

        let from_datex_fields_inner = match fields_type {
            FieldsType::Named => quote! {
                #variant_name => {
                    let mut map: Map = value.try_into()?;
                    map.ensure_only_properties(&[
                        #(#field_names),*
                    ])
                    .map_err(|err| TryFromDatexValueError(err.to_string()))?;

                    #helper_struct_ident {
                        #(#from_datex_fields),*
                    }.into()
                }
            },
            FieldsType::Transparent => {
                let from_field = from_datex_fields.first().unwrap();
                quote! {
                    #variant_name => {
                        #helper_struct_ident (
                            #from_field
                        ).into()
                    }
                }
            }
            FieldsType::Unnamed => {
                quote! {
                    #variant_name => {
                        let mut list: List = value.try_into()?;

                        #helper_struct_ident(
                            #(#from_datex_fields),*
                        ).into()
                    }
                }
            }
            FieldsType::Unit => quote! {
                #variant_name => {
                    if !value.is_null() {
                        return Err(TryFromDatexValueError(format!("Unexpected value, expected null")));
                    }
                    #ident::#variant_ident
                }
            },
        };

        let datex_type_for_variant = match datex_type {
            None => quote! {
                Type::Alias(TypeDefinition::TaggedType(TaggedTypeDefinition {
                    tag: #variant_name.to_string(),
                    ty: None,
                }).into())
            },
            Some(type_definition) => quote! {
                Type::Alias(TypeDefinition::TaggedType(TaggedTypeDefinition {
                    tag: #variant_name.to_string(),
                    ty: Some(Box::new(#type_definition)),
                }).into())
            },
        };

        variants_into_datex_fields.push(into_datex_fields_inner);
        variants_from_datex_fields.push(from_datex_fields_inner);
        variants_datex_types.push(datex_type_for_variant);
    }

    let into_datex_fields_inner = quote! {
        match &value {
            #(#variants_into_datex_fields),*
        }
    };

    let helpers = quote! {
        #(#helper_structs)*
    };

    let from_datex_fields_inner = quote! {
        match &value.custom_type {
            Some(TypeDefinition::TaggedType(TaggedTypeDefinition {tag, ..})) => {
                match tag.as_str() {
                    #(#variants_from_datex_fields),*
                    tag => return Err(TryFromDatexValueError(format!("Unexpected tag: {}", tag)))
                }
            }
            _ => return Err(TryFromDatexValueError("Expected tagged value".to_string())),
        }
    };

    let type_definition = quote! {
        Type::Alias(
            TypeDefinition::Union(UnionTypeDefinition(vec![
                #(#variants_datex_types),*
            ])).into()
        )
    };

    DeriveData {
        is_fallible_serialization,
        into_datex_fields_inner,
        from_datex_fields_inner,
        datex_type: type_definition,
        helpers: Some(helpers),
    }
}

struct FieldDeriveData {
    is_fallible_serialization: bool,
    fields_type: FieldsType,
    into_datex_fields: Vec<TokenStream>,
    from_datex_fields: Vec<TokenStream>,
    datex_type: Option<TokenStream>,
    field_names: Vec<String>,
}

fn generate_enum_helper_structs(
    fields: &Fields,
    ident: &Ident,
    variant_ident: &Ident,
    helper_struct_ident: &Ident,
) -> TokenStream {
    match fields {
        Fields::Named(fields_named) => {
            let fields = &fields_named.named;

            let field_idents = fields.iter().map(|f| f.ident.as_ref().unwrap());
            let field_idents2 = field_idents.clone();
            let field_idents3 = field_idents.clone();

            quote! {
                pub struct #helper_struct_ident {
                    #fields
                }

                impl From<#ident> for #helper_struct_ident {
                    fn from(value: #ident) -> Self {
                        match value {
                            #ident::#variant_ident {
                                #(#field_idents),*
                            } => Self {
                                #(
                                    #field_idents2
                                ),*
                            },

                            _ => unreachable!(),
                        }
                    }
                }

                impl From<#helper_struct_ident> for #ident {
                    fn from(value: #helper_struct_ident) -> Self {
                        #ident::#variant_ident {
                            #(
                                #field_idents3: value.#field_idents3
                            ),*
                        }
                    }
                }
            }
        }

        Fields::Unnamed(fields_unnamed) => {
            let fields = &fields_unnamed.unnamed;

            let field_idents = fields
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("v{i}"));

            let field_idents2 = fields
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("v{i}"));

            let field_indexes = (0..fields.len()).map(syn::Index::from);

            quote! {
                pub struct #helper_struct_ident(
                    #fields
                );

                impl From<#ident> for #helper_struct_ident {
                    fn from(value: #ident) -> Self {
                        match value {
                            #ident::#variant_ident(
                                #(#field_idents),*
                            ) => Self(
                                #(#field_idents2),*
                            ),

                            _ => unreachable!()
                        }
                    }
                }

                impl From<#helper_struct_ident> for #ident {
                    fn from(value: #helper_struct_ident) -> Self {
                        #ident::#variant_ident(
                            #(
                                value.#field_indexes
                            ),*
                        )
                    }
                }
            }
        }

        Fields::Unit => {
            quote! {}
        }
    }
}

fn derive_fields(fields: &Fields) -> FieldDeriveData {
    // serialization is only infallible if no serde fields or only serde fields with datex(serde_infallible)
    let mut is_fallible_serialization = false;

    let mut into_datex_fields: Vec<TokenStream> = vec![];
    let mut from_datex_fields: Vec<TokenStream> = vec![];
    let mut field_types: Vec<TokenStream> = vec![];
    let mut field_names: Vec<String> = vec![];

    let is_single_type = fields.len() == 1;

    // Iterate over the fields of the struct
    for (index, field) in fields.iter().enumerate() {
        let field_attributes = parse_field_attributes(&field.attrs);
        if !field_attributes.datex_skip
            && field_attributes.serde_mode == SerdeMode::Fallible
        {
            is_fallible_serialization = true;
        }

        let from_value_container_function = match field_attributes.serde_mode {
            SerdeMode::None => {
                quote! {
                    DatexValueContainerProxyDeserialize::try_from_value_container
                }
            }
            SerdeMode::Fallible | SerdeMode::Infallible => {
                quote! {
                    try_serde_from_value_container
                }
            }
        };

        match &field.ident {
            // struct with named fields
            Some(field_ident) => {
                let field_name = field_attributes
                    .datex_rename
                    .unwrap_or_else(|| field_ident.to_string());

                if field_attributes.datex_skip {
                    from_datex_fields.push(quote! {
                        #field_ident: ::core::default::Default::default()
                    });
                    continue;
                }

                let field_into = generate_field_conversion_code(
                    &field_attributes.serde_mode,
                    field_ident,
                    field_name.clone(),
                );

                let field_type = &field.ty;

                into_datex_fields.push(quote! {
                    (
                        #field_name.to_string(),
                        #field_into
                    )
                });
                if field_names.contains(&field_name) {
                    // This can happen, if the user makes invalid use of #[datex(rename = "whatever")]
                    panic!(
                        "Duplicate field name after renaming: {}",
                        field_name
                    );
                }

                field_names.push(field_name.clone());

                let field_from = match field_attributes.serde_mode {
                    SerdeMode::None => {
                        if field_attributes.datex_default {
                            quote! {
                                match unsafe { map.try_delete_unsafe(#field_name) } {
                                    Ok(value_container) => {
                                        DatexValueContainerProxyDeserialize::try_from_value_container(
                                            value_container
                                        )?
                                    }
                                    Err(_) => ::core::default::Default::default(),
                                }
                            }
                        } else {
                            quote! {
                                DatexValueContainerProxyDeserialize::try_from_map_property(
                                    unsafe {
                                        map.try_delete_unsafe(#field_name)
                                    }
                                )?
                            }
                        }
                    }

                    SerdeMode::Fallible | SerdeMode::Infallible => {
                        if field_attributes.datex_default {
                            quote! {
                                match unsafe { map.try_delete_unsafe(#field_name) } {
                                    Ok(value_container) => {
                                        try_serde_from_value_container(value_container)?
                                    }
                                    Err(_) => ::core::default::Default::default(),
                                }
                            }
                        } else {
                            quote! {
                                try_serde_from_value_container(
                                    unsafe {
                                        map.try_delete_unsafe(#field_name)
                                            .map_err(|err| {
                                                TryFromDatexValueError(err.to_string())
                                            })?
                                    }
                                )?
                            }
                        }
                    }
                };

                from_datex_fields.push(quote! {
                    #field_ident: #field_from
                });

                field_types.push(generate_named_field_type_code(
                    &field_attributes.serde_mode,
                    &field_name,
                    field_type,
                ));
            }

            // tuple struct or unit struct
            None => {
                if field_attributes.datex_skip {
                    panic!("datex(skip) is only supported on named fields");
                }

                if field_attributes.datex_default {
                    panic!("datex(default) is only supported on named fields");
                }

                let field_index = syn::Index::from(index);
                let field_into = generate_field_conversion_code(
                    &field_attributes.serde_mode,
                    &field_index,
                    field_index.index.to_string(),
                );
                let field_type = &field.ty;

                into_datex_fields.push(field_into);

                if is_single_type {
                    from_datex_fields.push(quote! {
                        #from_value_container_function(
                            ValueContainer::Local(value)
                        )?
                    });
                } else {
                    from_datex_fields.push(quote! {
                        #from_value_container_function(
                            list.try_set(#index as i64, ValueContainer::from(Value::null()))
                                .map_err(|err| TryFromDatexValueError(err.to_string()))?
                        )?
                    });
                }

                field_types.push(generate_unnamed_field_type_code(
                    &field_attributes.serde_mode,
                    field_type,
                ));
            }
        }
    }

    let fields_type = FieldsType::from(fields);

    let datex_type = match fields_type {
        FieldsType::Unit => None,
        FieldsType::Named => Some(quote! {
            Type::Alias(TypeDefinition::Map(MapTypeDefinition(vec![
                #(#field_types),*
            ])).into())
        }),
        FieldsType::Unnamed => Some(quote! {
            Type::Alias(TypeDefinition::List(ListTypeDefinition(vec![
                #(#field_types),*
            ])).into())
        }),
        FieldsType::Transparent => Some(field_types.remove(0)),
    };

    FieldDeriveData {
        is_fallible_serialization,
        fields_type,
        into_datex_fields,
        from_datex_fields,
        datex_type,
        field_names,
    }
}

fn parse_field_attributes(attrs: &[Attribute]) -> FieldAttributes {
    let mut serde_mode = SerdeMode::None;
    let mut datex_rename = None;
    let mut datex_default = false;
    let mut datex_skip = false;

    // find datex(serde) or datex(serde_infallible) attribute
    for attr in attrs {
        if attr.path().is_ident("datex")
            && let Meta::List(meta_list) = &attr.meta
        {
            for nested in meta_list
                .parse_args_with(
                    Punctuated::<Meta, Token![,]>::parse_terminated,
                )
                .unwrap()
            {
                match nested {
                    Meta::Path(path) => {
                        if path.is_ident("serde") {
                            if matches!(serde_mode, SerdeMode::Infallible) {
                                panic!(
                                    "Cannot use both datex(serde) and datex(serde_infallible)"
                                );
                            }
                            serde_mode = SerdeMode::Fallible;
                        } else if path.is_ident("serde_infallible") {
                            if matches!(serde_mode, SerdeMode::Fallible) {
                                panic!(
                                    "Cannot use both datex(serde) and datex(serde_infallible)"
                                );
                            }
                            serde_mode = SerdeMode::Infallible;
                        } else if path.is_ident("default") {
                            datex_default = true;
                        } else if path.is_ident("skip") {
                            datex_skip = true;
                        } else {
                            panic!(
                                "Unknown datex field attribute: {}",
                                path.get_ident().unwrap()
                            );
                        }
                    }

                    Meta::NameValue(nv) if nv.path.is_ident("rename") => {
                        let value = match &nv.value {
                            syn::Expr::Lit(expr_lit) => {
                                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                    lit_str.value()
                                } else {
                                    panic!(
                                        "datex(rename = ...) must be a string"
                                    )
                                }
                            }
                            _ => panic!(
                                "datex(rename = ...) must be a string literal"
                            ),
                        };

                        datex_rename = Some(value);
                    }

                    _ => {}
                }
            }
        }
    }

    if datex_skip && datex_default {
        panic!("Cannot use both datex(skip) and datex(default)");
    }

    FieldAttributes {
        serde_mode,
        datex_rename,
        datex_default,
        datex_skip,
    }
}

fn parse_string_attribute(
    name_value: &syn::MetaNameValue,
    attribute_name: &str,
) -> String {
    match &name_value.value {
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Str(lit_str) => lit_str.value(),
            _ => {
                panic!("datex({attribute_name} = ...) must be a string literal")
            }
        },
        _ => panic!("datex({attribute_name} = ...) must be a string literal"),
    }
}

fn parse_doc_comments(attrs: &[Attribute]) -> Option<String> {
    let docs = attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }
            let Meta::NameValue(name_value) = &attr.meta else {
                return None;
            };
            let syn::Expr::Lit(expr_lit) = &name_value.value else {
                return None;
            };
            let syn::Lit::Str(lit_str) = &expr_lit.lit else {
                return None;
            };
            Some(lit_str.value().trim_start().to_string())
        })
        .collect::<Vec<_>>();
    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}

fn parse_top_level_attributes(attrs: &[Attribute]) -> TopLevelAttributes {
    let mut force_datex_core_namespace = false;
    let mut datex_name = None;
    let mut export = false;
    let mut namespace = None;

    for attr in attrs {
        if !attr.path().is_ident("datex") {
            continue;
        }
        let Meta::List(meta_list) = &attr.meta else {
            continue;
        };
        let nested = meta_list
            .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .unwrap_or_else(|error| {
                panic!("Invalid #[datex(...)] attribute: {error}")
            });

        for meta in nested {
            match meta {
                Meta::Path(path)
                    if path.is_ident("_force_datex_core_namespace") =>
                {
                    force_datex_core_namespace = true;
                }
                Meta::Path(path) if path.is_ident("export") => {
                    export = true;
                }

                Meta::NameValue(name_value)
                    if name_value.path.is_ident("name") =>
                {
                    if datex_name.is_some() {
                        panic!("datex(name = ...) must only be specified once");
                    }
                    datex_name =
                        Some(parse_string_attribute(&name_value, "name"));
                }

                Meta::NameValue(name_value)
                    if name_value.path.is_ident("namespace") =>
                {
                    if namespace.is_some() {
                        panic!(
                            "datex(namespace = ...) must only be specified once"
                        );
                    }
                    namespace =
                        Some(parse_string_attribute(&name_value, "namespace"));
                    export = true;
                }
                _ => {}
            }
        }
    }

    TopLevelAttributes {
        force_datex_core_namespace,
        datex_name,
        export,
        namespace,
        docs: parse_doc_comments(attrs),
    }
}

/// Generate the code to convert a field to a ValueContainer, depending on the serde mode of the field
fn generate_field_conversion_code<T: ToTokens>(
    serde_mode: &SerdeMode,
    field_identifier: T,
    field_name: String,
) -> TokenStream {
    match serde_mode {
        // no serde or infallible serde, provide/assume DatexValueContainerProxyInfallibleSerialize
        SerdeMode::None => {
            quote! {
                DatexValueContainerProxyInfallibleSerialize::to_value_container(value.#field_identifier)
            }
        }
        // Map serde fields and propagate the error if the serialization fails
        SerdeMode::Fallible => {
            quote! {
                try_serde_to_value_container(value.#field_identifier)?
            }
        }
        // Allow serde fields that only default implement DatexValueContainerProxySerialize
        // but panic if the serialization fails, since the user explicitly guarantees that it won't fail
        SerdeMode::Infallible => {
            quote! {
                try_serde_to_value_container(value.#field_identifier).unwrap_or_else(|err| panic!("Serialization of field '{}' marked with (serde_infallible) failed: {:?}", #field_name, err))
            }
        }
    }
}

fn generate_named_field_type_code(
    serde_mode: &SerdeMode,
    field_name: &String,
    field_type: &syn::Type,
) -> TokenStream {
    match serde_mode {
        // no serde or infallible serde, provide/assume DatexValueContainerProxyInfallibleSerialize
        SerdeMode::None => {
            quote! {
                (
                    Type::Alias(TypeDefinition::Literal(LiteralTypeDefinition::Text(#field_name.into())).into()),
                    <#field_type as DatexProxyTypes>::datex_type(memory)
                )
            }
        }
        // Cannot infer type
        SerdeMode::Fallible | SerdeMode::Infallible => {
            quote! {
                (
                    Type::Alias(TypeDefinition::Literal(LiteralTypeDefinition::Text(#field_name.into())).into()),
                    Type::Alias(TypeDefinition::CoreType(CoreLibTypeId::Base(CoreLibBaseTypeId::Unknown)).into())
                )
            }
        }
    }
}

fn generate_unnamed_field_type_code(
    serde_mode: &SerdeMode,
    field_type: &syn::Type,
) -> TokenStream {
    match serde_mode {
        // no serde or infallible serde, provide/assume DatexValueContainerProxyInfallibleSerialize
        SerdeMode::None => {
            quote! {
                <#field_type as DatexProxyTypes>::datex_type(memory)
            }
        }
        // Cannot infer type
        SerdeMode::Fallible | SerdeMode::Infallible => {
            quote! {
               Type::Alias(TypeDefinition::CoreType(CoreLibTypeId::Base(CoreLibBaseTypeId::Unknown)).into())
            }
        }
    }
}

fn get_datex_core_crate_name() -> Ident {
    // otherwise, find the crate name of datex_core and use it as an identifier
    let found = crate_name("datex-core").unwrap();
    match found {
        FoundCrate::Itself => format_ident!("crate"),
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    }
}
