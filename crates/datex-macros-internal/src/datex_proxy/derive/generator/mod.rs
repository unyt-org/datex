use std::str::FromStr;

use crate::{
    datex_proxy::{
        data::StructureData,
        generator::{
            classification::generate_classification,
            convert_parts::generate_convert_parts,
            datex_expression_data::generate_datex_expression_data,
            datex_hash::generate_datex_hash,
            datex_native::generate_datex_native,
            datex_type::{generate_core_lib_type_id, generate_datex_type},
            to_instructions::generate_to_instructions,
            try_from_core_value::generate_try_from_core_value,
            value_access::generate_value_access,
        },
    },
    utils::get_datex_core_crate_name,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::PathSegment;
pub mod classification;
mod convert_parts;
mod datex_expression_data;
pub mod datex_hash;
mod datex_native;
mod datex_type;
mod to_instructions;
pub mod try_from_core_value;
pub mod value_access;

/// Generates the code for the derive macro based on the provided structure data.
pub fn generate_derive_code(structure_data: StructureData) -> TokenStream {
    // generate trait impls
    let datex_native = generate_datex_native(&structure_data);
    let convert_parts = generate_convert_parts(&structure_data);
    let datex_type = generate_datex_type(&structure_data);
    let core_lib_type_id = generate_core_lib_type_id(&structure_data);
    let try_from_core_value = generate_try_from_core_value(&structure_data);
    let classification = generate_classification(&structure_data);
    let value_access = generate_value_access(&structure_data);
    let datex_hash = generate_datex_hash(&structure_data);
    let to_instructions = generate_to_instructions(&structure_data);

    let datex_expression_data =
        cfg_select! {
            feature = "ast" => generate_datex_expression_data(&structure_data),
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
        const _: () = {
            use #datex_core_crate_name::preludes::derive::*;
            #datex_native
            #convert_parts
            #datex_type
            #core_lib_type_id
            #try_from_core_value
            #classification
            #value_access
            #datex_hash
            #to_instructions
            #datex_expression_data
        };
    }
}
