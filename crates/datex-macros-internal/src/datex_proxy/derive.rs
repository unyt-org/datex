use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Meta, Token,
    punctuated::Punctuated,
};

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
}

impl From<&Fields> for FieldsType {
    fn from(fields: &Fields) -> Self {
        match &fields {
            Fields::Unit => FieldsType::Unit,
            Fields::Named(_) => FieldsType::Named,
            Fields::Unnamed(_) => FieldsType::Unnamed,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FieldAttributes {
    serde_mode: SerdeMode,
    datex_rename: Option<String>,
}

pub struct DeriveData {
    is_fallible_serialization: bool,
    from_datex_fields_inner: TokenStream,
    into_datex_fields_inner: TokenStream,
    helpers: Option<TokenStream>,
}

/// Derive implementation for the [Datex] derive macro.
pub fn derive(input: DeriveInput) -> TokenStream {
    let datex_core_crate_name = get_datex_core_crate_name(&input.attrs);

    let DeriveData {
        into_datex_fields_inner,
        from_datex_fields_inner,
        is_fallible_serialization,
        helpers,
    } = match input.data {
        Data::Struct(data_struct) => derive_struct(data_struct, &input.ident),
        Data::Enum(data_enum) => derive_enum(data_enum, &input.ident),
        _ => unimplemented!(),
    };

    let ident = input.ident;

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
                    serde_compat::{
                        try_serde_to_value_container,
                        try_serde_from_value_container
                    }
                },
                values::value_container::ValueContainer,
                values::value::Value,
                values::core_value::CoreValue,
                values::core_values::map::Map,
                values::core_values::list::List,
                types::type_definition::TypeDefinition,
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
    } = derive_fields(&data_struct.fields);

    let into_datex_fields_inner = match fields_type {
        FieldsType::Named => quote! {
            Value::from(Map::from(vec![
                #(#into_datex_fields),*
            ]))
        },
        FieldsType::Unnamed => {
            if into_datex_fields.len() == 1 {
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
            } else {
                quote! {
                    Value::from(List::from(vec![
                        #(#into_datex_fields),*
                    ]))
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

            #ident {
                #(#from_datex_fields),*
            }
        }},
        FieldsType::Unnamed => {
            if from_datex_fields.len() == 1 {
                let from_field = from_datex_fields.first().unwrap();
                quote! {{
                    #ident(#from_field)
                }}
            } else {
                quote! {{
                    let mut list: List = value.try_into()?;

                    #ident(
                        #(#from_datex_fields),*
                    )
                }}
            }
        }
        FieldsType::Unit => quote! {
            if !value.is_null() {
                return Err(TryFromDatexValueError(format!("Unexpected value, expected null")));
            }
            #ident
        },
    };

    DeriveData {
        is_fallible_serialization,
        into_datex_fields_inner,
        from_datex_fields_inner,
        helpers: None,
    }
}

fn derive_enum(data_enum: DataEnum, ident: &Ident) -> DeriveData {
    // serialization is only infallible if the enum has no serde fields or only serde fields with datex(serde_infallible)
    let mut is_fallible_serialization = false;

    let mut into_datex_fields_inners = Vec::<TokenStream>::new();
    let mut from_datex_fields_inners = Vec::<TokenStream>::new();

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
        } = derive_fields(&variant.fields);

        // if any variant is fallible, mark as fallible
        if variant_is_fallible_serialization {
            is_fallible_serialization = true;
        }

        let into_datex_fields_inner = match fields_type {
            FieldsType::Named => quote! {
                #ident::#variant_ident {..} => {
                    let value: #helper_struct_ident = value.into();
                    let map = Map::from(vec![
                        #(#into_datex_fields),*
                    ]);
                    Value {
                        inner: CoreValue::Map(map),
                        custom_type: Some(TypeDefinition::TaggedType {
                            tag: #variant_name.to_string(),
                            ty: None,
                        }),
                    }
                }
            },
            FieldsType::Unnamed => {
                if into_datex_fields.len() == 1 {
                    let into_field = into_datex_fields.first().unwrap();
                    quote! {
                        #ident::#variant_ident {..} => {
                            let value: #helper_struct_ident = value.into();
                            let container = #into_field;
                            if let ValueContainer::Local(Value {custom_type: None, inner}) = container {
                                Value {
                                    inner,
                                    custom_type: Some(TypeDefinition::TaggedType {
                                        tag: #variant_name.to_string(),
                                        ty: None,
                                    }),
                                }
                            }
                            else {
                                unreachable!("Expected ValueContainer::Local without custom type");
                            }
                        }
                    }
                } else {
                    quote! {
                        #ident::#variant_ident (..) => {
                            let value: #helper_struct_ident = value.into();
                            let list = List::from(vec![
                                #(#into_datex_fields),*
                            ]);
                            Value {
                                inner: CoreValue::List(list),
                                custom_type: Some(TypeDefinition::TaggedType {
                                    tag: #variant_name.to_string(),
                                    ty: None,
                                }),
                            }
                        }
                    }
                }
            }
            FieldsType::Unit => quote! {
                #ident::#variant_ident => {
                    Value {
                        inner: CoreValue::Null,
                        custom_type: Some(TypeDefinition::TaggedType {
                            tag: #variant_name.to_string(),
                            ty: None,
                        }),
                    }
                }
            },
        };

        let from_datex_fields_inner = match &variant.fields {
            Fields::Named(_) => quote! {
                #variant_name => {
                    let mut map: Map = value.try_into()?;

                    #helper_struct_ident {
                        #(#from_datex_fields),*
                    }.into()
                }
            },
            Fields::Unnamed(_) => {
                if from_datex_fields.len() == 1 {
                    let from_field = from_datex_fields.first().unwrap();
                    quote! {
                        #variant_name => {
                            #helper_struct_ident (
                                #from_field
                            ).into()
                        }
                    }
                } else {
                    quote! {
                        #variant_name => {
                            let mut list: List = value.try_into()?;

                            #helper_struct_ident(
                                #(#from_datex_fields),*
                            ).into()
                        }
                    }
                }
            }
            Fields::Unit => quote! {
                #variant_name => {
                    if !value.is_null() {
                        return Err(TryFromDatexValueError(format!("Unexpected value, expected null")));
                    }
                    #ident::#variant_ident
                }
            },
        };

        into_datex_fields_inners.push(into_datex_fields_inner);
        from_datex_fields_inners.push(from_datex_fields_inner);
    }

    let into_datex_fields_inner = quote! {
        match &value {
            #(#into_datex_fields_inners),*
        }
    };

    let helpers = quote! {
        #(#helper_structs)*
    };

    let from_datex_fields_inner = quote! {
        match &value.custom_type {
            Some(TypeDefinition::TaggedType {tag, ..}) => {
                match tag.as_str() {
                    #(#from_datex_fields_inners),*
                    tag => return Err(TryFromDatexValueError(format!("Unexpected tag: {}", tag)))
                }
            }
            _ => return Err(TryFromDatexValueError("Expected tagged value".to_string())),
        }
    };

    DeriveData {
        is_fallible_serialization,
        into_datex_fields_inner,
        from_datex_fields_inner,
        helpers: Some(helpers),
    }
}

struct FieldDeriveData {
    is_fallible_serialization: bool,
    fields_type: FieldsType,
    into_datex_fields: Vec<TokenStream>,
    from_datex_fields: Vec<TokenStream>,
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

    let is_new_type = fields.len() == 1;

    // Iterate over the fields of the struct
    for (index, field) in fields.iter().enumerate() {
        let field_attributes = parse_field_attributes(&field.attrs);
        if field_attributes.serde_mode == SerdeMode::Fallible {
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
                let field_into = generate_field_conversion_code(
                    field_attributes.serde_mode,
                    field_ident,
                    field_name.clone(),
                );

                into_datex_fields.push(quote! {
                    (
                        #field_name.to_string(),
                        #field_into
                    )
                });

                from_datex_fields.push(quote! {
                    #field_ident: #from_value_container_function(
                            unsafe {
                                map.try_delete_unsafe(#field_name)
                                    .map_err(|err| TryFromDatexValueError(err.to_string()))?
                            }
                        )?
                });
            }

            // tuple struct or unit struct
            None => {
                let field_index = syn::Index::from(index);
                let field_into = generate_field_conversion_code(
                    field_attributes.serde_mode,
                    &field_index,
                    field_index.index.to_string(),
                );

                into_datex_fields.push(field_into);

                if is_new_type {
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
            }
        }
    }

    FieldDeriveData {
        is_fallible_serialization,
        fields_type: FieldsType::from(fields),
        into_datex_fields,
        from_datex_fields,
    }
}

fn parse_field_attributes(attrs: &[Attribute]) -> FieldAttributes {
    let mut serde_mode = SerdeMode::None;
    let mut datex_rename = None;

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

    FieldAttributes {
        serde_mode,
        datex_rename,
    }
}

/// Generate the code to convert a field to a ValueContainer, depending on the serde mode of the field
fn generate_field_conversion_code<T: ToTokens>(
    serde_mode: SerdeMode,
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

fn get_datex_core_crate_name(_attrs: &[Attribute]) -> Ident {
    // otherwise, find the crate name of datex_core and use it as an identifier
    let found = crate_name("datex-core").unwrap();
    match found {
        FoundCrate::Itself => format_ident!("crate"),
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    }
}
