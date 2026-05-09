use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Meta, Token};
use syn::punctuated::Punctuated;

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
    Infallible
}

/// Derive implementation for the [Datex] derive macro.
pub fn derive(input: DeriveInput) -> TokenStream {
    match input.data {
        Data::Struct(data_struct) => derive_struct(data_struct, input.ident, input.attrs),
        Data::Enum(data_enum) => derive_enum(data_enum),
        _ => unimplemented!(),
    }
}

/// Derive implementation for structs
fn derive_struct(data_struct: DataStruct, ident: Ident, attrs: Vec<Attribute>) -> TokenStream {
    let serde_mode = get_serde_mode(&attrs);
    let mut into_datex_fields: Vec<TokenStream> = vec![];
    let mut from_datex_fields: Vec<TokenStream> = vec![];

    let datex_core_crate_name = get_datex_core_crate_name(&attrs);

    let has_named_fields = matches!(data_struct.fields, syn::Fields::Named(_));

    // Iterate over the fields of the struct
    for (index, field) in data_struct.fields.iter().enumerate() {
        match &field.ident {
            // struct with named fields
            Some(field_ident) => {
                let field_name = field_ident.to_string();

                let field_into = match serde_mode {
                    // no serde or infallible serde, provide/assume DatexProxyInfallibleSerialize
                    SerdeMode::None => {
                        quote! {
                            DatexProxyInfallibleSerialize::to_value_container(value.#field_ident)
                        }
                    },
                    // Allow serde fields that only default implement DatexProxySerialize
                    // and propagate the error if the serialization fails
                    SerdeMode::Fallible => {
                        quote! {
                            DatexProxySerialize::try_to_value_container(value.#field_ident).map_err(|_| ())?,
                        }
                    },
                    // Allow serde fields that only default implement DatexProxySerialize
                    // but panic if the serialization fails, since the user explicitly guarantees that it won't fail
                    SerdeMode::Infallible => {
                        quote! {
                            DatexProxySerialize::try_to_value_container(value.#field_ident).expect("Serde serialization for Datex value with datex(serde_infallible)")
                        }
                    },
                };

                into_datex_fields.push(quote! {
                    (
                        #field_name.to_string(),
                        #field_into
                    ),
                });

                from_datex_fields.push(quote! {
                    #field_ident: DatexProxyDeserialize::try_from_value_container(
                            map.get(#field_name)
                                .map_err(|_| ())?
                                .clone()
                        )
                        .map_err(|_| ())?,
                });
            }

            // tuple struct or unit struct
            None => {
                let field_index = syn::Index::from(index);

                into_datex_fields.push(quote! {
                    DatexProxyInfallibleSerialize::to_value_container(value.#field_index)
                        .map_err(|_| ())?,
                });

                from_datex_fields.push(quote! {
                    DatexProxyDeserialize::try_from_value_container(
                            list.get(#index)
                                .map_err(|_| ())?
                                .clone()
                        )
                        .map_err(|_| ())?,
                });
            }
        }
    }

    let into_datex_fields_inner = if has_named_fields {
        quote! {
            Map::from(vec![
                #(#into_datex_fields)*
            ])
        }
    } else {
        quote! {
            List::from(vec![
                #(#into_datex_fields)*
            ])
        }
    };

    let from_datex_fields_inner = if has_named_fields {
        quote! {{
            let map: Map = value.try_into().map_err(|_| ())?;

            #ident {
                #(#from_datex_fields)*
            }
        }}
    } else {
        quote! {{
            let list: List = value.try_into().map_err(|_| ())?;

            #ident(
                #(#from_datex_fields)*
            )
        }}
    };

    let serialize = match serde_mode {
        // no serde or infallible serde, provide/assume DatexProxyInfallibleSerialize
        SerdeMode::None | SerdeMode::Infallible=> {
            quote! {
                #[automatically_derived]
                impl From<#ident> for Value {
                    fn from(value: #ident) -> Self {
                        Value::from(#into_datex_fields_inner)
                    }
                }

                #[automatically_derived]
                impl DatexProxyInfallibleSerialize for #ident {
                    fn to_value_container(self) -> ValueContainer {
                       ValueContainer::Local(self.into())
                    }
                }

                #[automatically_derived]
                impl DatexProxySerialize for #ident {
                    fn try_to_value_container(self) -> Result<ValueContainer, ()> {
                        Ok(ValueContainer::Local(self.into()))
                    }
                }
            }
        },
        SerdeMode::Fallible => {
            quote! {
                #[automatically_derived]
                impl TryFrom<#ident> for Value {
                    type Error = ();

                    fn try_from(value: #ident) -> Result<Self, Self::Error> {
                        Ok(Value::from(#into_datex_fields_inner))
                    }
                }

                #[automatically_derived]
                impl TryFrom<#ident> for ValueContainer {
                    type Error = ();

                    fn try_from(value: #ident) -> Result<Self, Self::Error> {
                        Ok(ValueContainer::Local(Value::from(#into_datex_fields_inner)))
                    }
                }

                #[automatically_derived]
                impl DatexProxySerialize for #ident {
                    fn try_to_value_container(self) -> Result<ValueContainer, ()> {
                        self.try_into().map(|value| ValueContainer::Local(value))
                    }
                }
            }
        },
    };

    quote! {
        const _: () = {
            use #datex_core_crate_name::{
                datex_proxy::{DatexProxy, DatexProxyInfallibleSerialize, DatexProxySerialize, DatexProxyDeserialize},
                values::value_container::ValueContainer,
                values::value::Value,
                values::core_values::map::Map,
                values::core_values::list::List,
            };

            #[automatically_derived]
            impl DatexProxy for #ident {}

            #serialize

            #[automatically_derived]
            impl DatexProxyDeserialize for #ident {
                fn try_from_value_container(
                    value: ValueContainer,
                ) -> Result<Self, ()> {
                   value.try_into()
                }
            }

            #[automatically_derived]
            impl TryFrom<Value> for #ident {
                type Error = ();

                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    Ok(#from_datex_fields_inner)
                }
            }

            #[automatically_derived]
            impl TryFrom<ValueContainer> for #ident {
                type Error = ();

                fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
                    match value {
                        ValueContainer::Local(value) => value.try_into(),
                        _ => Err(()),
                    }
                }
            }
        };
    }
}

fn get_serde_mode(attrs: &[Attribute]) -> SerdeMode {
    let mut serde_mode = SerdeMode::None;

    // find datex(allow_serde) or datex(allow_serde_infallible) attribute
    for attr in attrs {
        if attr.path().is_ident("datex") && let Meta::List(meta_list) = &attr.meta {
            for nested in meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap() {
                if let Meta::Path(path) = nested {
                    if path.is_ident("allow_serde") {
                        if let SerdeMode::Infallible = serde_mode {
                            panic!("Cannot use both datex(allow_serde) and datex(allow_serde_infallible) on the same struct");
                        }
                        serde_mode = SerdeMode::Fallible;
                    } else if path.is_ident("allow_serde_infallible") {
                        if let SerdeMode::Fallible = serde_mode {
                            panic!("Cannot use both datex(allow_serde) and datex(allow_serde_infallible) on the same struct");
                        }
                        serde_mode = SerdeMode::Infallible;
                    }
                }
            }
        }
    }

    serde_mode
}

fn get_datex_core_crate_name(attrs: &[Attribute]) -> Ident {
    // find datex(internal) attribute -> use crate:: identifier
    for attr in attrs {
        if attr.path().is_ident("datex") && let Meta::List(meta_list) = &attr.meta {
            for nested in meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap() {
                if let Meta::Path(path) = nested && path.is_ident("internal") {
                    return format_ident!("crate");
                }
            }
        }
    }

    // otherwise, find the crate name of datex_core and use it as an identifier
    let found = crate_name("datex-core").unwrap();
    match found {
        FoundCrate::Itself => format_ident!("crate"),
        FoundCrate::Name(name) => {
            Ident::new(&name, Span::call_site())
        }
    }
}


fn derive_enum(data_enum: DataEnum) -> TokenStream {
    todo!()
}
