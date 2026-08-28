use std::{path::PathBuf, str::FromStr};

use crate::{
    datex_proxy::{
        data::{
            EnumVariant, Field, Fields, NamedField, Structure, StructureData,
            TypeKind,
        },
        generator::{
            datex_expression_data::generate_datex_expression_data,
            datex_native::generate_datex_native,
            datex_proxy_type::generate_datex_proxy_types,
        },
    },
    utils::get_datex_core_crate_name,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::PathSegment;

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
    let datex_core_crate_name =
        if structure_data.attributes.force_datex_core_namespace {
            PathSegment::from(Ident::new("datex_core", Span::call_site()))
                .into()
        } else {
            get_datex_core_crate_name()
        };

    quote! {
        use #datex_core_crate_name::preludes::derive::*;
        #datex_native

        #datex_types

        #datex_expression_data
    }
}
