use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DataEnum, DataStruct, DeriveInput};

pub fn derive(input: DeriveInput) -> TokenStream {
    match input.data {
        Data::Struct(data_struct) => derive_struct(
            data_struct,
            input.ident,
        ),
        Data::Enum(data_enum)  => derive_enum(data_enum),
        _ => unimplemented!(),
    }
}

pub fn derive_struct(data_struct: DataStruct, ident: Ident) -> TokenStream {
    let mod_ident = format_ident!("__{}_datex", ident.to_string().to_ascii_lowercase());


    let mut into_datex_fields: Vec<TokenStream> = vec![];
    let mut from_datex_fields: Vec<TokenStream> = vec![];


    let mut has_named_fields = false;

    for (index, field) in data_struct.fields.iter().enumerate() {
        into_datex_fields.push(match field.ident {
            Some(ref ident) => {
                has_named_fields = true;
                let ident_str = ident.to_string();
                quote! { (#ident_str.to_string(), value.#ident.try_into().map_err(|_| ())?), }
            },
            None => quote! { value.#index.try_into().map_err(|_| ())?, },
        });

        from_datex_fields.push(match field.ident {
            Some(ref ident) => {
                let ident_str = ident.to_string();
                quote! { #ident: map.get(#ident_str).map_err(|_| ())?.try_as().ok_or(())?, }
            },
            None => quote! { list.get(#index).map_err(|_| ())?.try_as().ok_or(())?, }
        });
    }

    let into_datex_fields_inner = match has_named_fields {
        true => quote! {Map::from(vec![#(#into_datex_fields)*])},
        false => quote! {List::from(vec![#(#into_datex_fields)*])}
    };

    let from_datex_fields_inner = match has_named_fields {
        true => quote! {{
            let map: Map = value.try_as().ok_or(())?;
            #ident { #(#from_datex_fields)* }
        }},
        false => quote! {{
            let list: List = value.try_as().ok_or(())?;
            #ident ( #(#from_datex_fields)* )
        }},
    };

    quote! {
        pub mod #mod_ident {
            use super::*;
            use datex_core::macro_utils::datex_proxy::DatexValueProxyWithSerde;
            use datex_core::values::value_container::ValueContainer;
            use datex_core::values::core_values::map::Map;
            use datex_core::values::core_values::list::List;

            impl DatexValueProxyWithSerde for #ident {}

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