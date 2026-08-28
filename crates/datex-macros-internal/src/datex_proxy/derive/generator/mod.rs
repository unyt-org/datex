use std::{path::PathBuf, str::FromStr};

use crate::datex_proxy::{
    data::{
        EnumVariant, Field, Fields, NamedField, Structure, StructureData,
        TypeKind,
    },
    generator::{
        datex_expression_data::generate_datex_expression_data,
        datex_native::generate_datex_native,
        datex_proxy_type::generate_datex_proxy_types,
    },
};
use proc_macro::Span;
use proc_macro2::TokenStream;
use quote::quote;

mod datex_expression_data;
mod datex_native;
mod datex_proxy_type;

/// Generates the code for the derive macro based on the provided structure data.
pub fn generate_derive_code(structure_data: StructureData) -> TokenStream {
    let datex_native = generate_datex_native(&structure_data);
    let datex_types = generate_datex_proxy_types(&structure_data);
    let datex_expression_data = cfg_select! {
        feature = "decompiler" => {
            generate_datex_expression_data(&structure_data)
        }
        _ => quote! {},
    };
    quote! {
        use #crate::preludes::derive::*;
        #datex_native

        #datex_types

        #datex_expression_data
    }
}
