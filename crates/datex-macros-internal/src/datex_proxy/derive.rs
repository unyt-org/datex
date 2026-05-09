use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DataEnum, DataStruct, DeriveInput};

/// Derive implementation for the [Datex] derive macro.
pub fn derive(input: DeriveInput) -> TokenStream {
    match input.data {
        Data::Struct(data_struct) => derive_struct(data_struct, input.ident),
        Data::Enum(data_enum) => derive_enum(data_enum),
        _ => unimplemented!(),
    }
}

/// Derive implementation for structs
pub fn derive_struct(data_struct: DataStruct, ident: Ident) -> TokenStream {
    let mod_ident =
        format_ident!("__{}_datex", ident.to_string().to_ascii_lowercase());

    let mut into_datex_fields: Vec<TokenStream> = vec![];
    let mut from_datex_fields: Vec<TokenStream> = vec![];

    let has_named_fields = matches!(data_struct.fields, syn::Fields::Named(_));

    // Iterate over the fields of the struct
    for (index, field) in data_struct.fields.iter().enumerate() {
        match &field.ident {
            // struct with named fields
            Some(field_ident) => {
                let field_name = field_ident.to_string();

                into_datex_fields.push(quote! {
                    (
                        #field_name.to_string(),
                        DatexProxy::datex_to_value_container(value.#field_ident)
                            .map_err(|_| ())?,
                    ),
                });

                from_datex_fields.push(quote! {
                    #field_ident: DatexProxy::datex_from_value_container(
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
                    DatexProxy::datex_to_value_container(value.#field_index)
                        .map_err(|_| ())?,
                });

                from_datex_fields.push(quote! {
                    DatexProxy::datex_from_value_container(
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
            let map: Map = value.try_as().ok_or(())?;

            #ident {
                #(#from_datex_fields)*
            }
        }}
    } else {
        quote! {{
            let list: List = value.try_as().ok_or(())?;

            #ident(
                #(#from_datex_fields)*
            )
        }}
    };

    quote! {
        pub mod #mod_ident {
            use super::*;

            use datex_core::macro_utils::datex_proxy::{
                DatexProxy,
            };

            use datex_core::values::value_container::ValueContainer;
            use datex_core::values::core_values::map::Map;
            use datex_core::values::core_values::list::List;

            impl DatexProxy for #ident {
                fn datex_to_value_container(self) -> Result<ValueContainer, ()> {
                    self.try_into()
                }

                fn datex_from_value_container(value: ValueContainer) -> Result<Self, ()> {
                    value.try_into()
                }
            }

            impl TryFrom<ValueContainer> for #ident {
                type Error = ();

                fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
                    Ok(#from_datex_fields_inner)
                }
            }

            impl TryFrom<#ident> for ValueContainer {
                type Error = ();

                fn try_from(value: #ident) -> Result<Self, Self::Error> {
                    Ok(ValueContainer::from(#into_datex_fields_inner))
                }
            }
        }
    }
}

pub fn derive_enum(data_enum: DataEnum) -> TokenStream {
    todo!()
}
