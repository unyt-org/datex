#![feature(box_patterns)]

use crate::datex_proxy::generate_impl_glue_code;
use proc_macro::TokenStream;
use syn::{Item, parse_macro_input};

mod bitfield_macros;
mod core_lib;
mod datex_proxy;
mod magic_rw;
mod utils;
mod value_macros;

#[proc_macro_derive(FromCoreValue)]
pub fn from_core_value_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    value_macros::from_core_value_derive_impl(input).into()
}

/// Unused and incomplete
#[proc_macro_derive(BitfieldSerde)]
pub fn derive_bitfield_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    bitfield_macros::derive_bitfield_serde(input).into()
}

#[proc_macro_derive(CoreLibString)]
pub fn core_lib_string(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    core_lib::derive_core_string(input).into()
}

/// This derive macro generates implementations of the Instruction trait for a struct or enum,
/// allowing it to be used as a DATEX instruction and converted from and to a Value
/// Usage:
/// ```txt
/// # use datex_macros_internal::Instruction;
///
/// #[repr(u8)]
/// enum Map {
///     A = 0,
///     B = 1,
///     C = 2,
/// }
///
/// #[repr(u8)]
/// #[derive(Instruction)]
/// enum MyInstruction {
///     #[magic(Map::A)]
///     Field1,
///     #[magic(Map::B)]
///     Field2,
/// }
/// ```
#[proc_macro_derive(Instruction, attributes(magic, instruction))]
pub fn derive_instruction(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    magic_rw::derive_instruction(input)
}

/// This derive macro generates implementations of the DatexValueContainerProxy trait for a struct or enum,
/// allowing it to be used as a DATEX value and converted from and to a Value
///
/// Usage:
/// ```rust
/// # use datex_macros_internal::Datex;
///
/// #[derive(Datex)]
/// struct MyStruct {
///     field1: String,
///     field2: u32,
/// }
/// ```
///
/// Structs and Enums that implement `Serialize` and `DeserializeOwned` from the `serde` crate can be used with this derive macro
/// by adding the `serde` attribute to the corresponding fields:
/// ```rust
/// # use datex_macros_internal::Datex;
/// # use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct SerdeStruct {
///     inner_field: String,
/// }
///
/// #[derive(Datex)]
/// struct MyStruct {
///     field1: String,
///     #[datex(serde)]
///     serde_field: SerdeStruct,
/// }
/// ```
/// Since the serialization of a struct with serde might fail, the generated code will only provide a try_into method to convert to ValueContainer,
/// which returns a Result that must be handled by the user.
///
/// Alternatively, if you can guarantee that the serialization will not fail, you can use the `serde_infallible` attribute,
/// which will generate an infallible into method to convert to ValueContainer, but will panic if the serialization fails:
///
/// ```rust
/// # use datex_macros_internal::Datex;
/// # use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct SerdeStruct {
///     inner_field: String,
/// }
///
/// #[derive(Datex)]
/// struct MyStruct {
///     field1: String,
///     #[datex(serde_infallible)]
///     serde_field: SerdeStruct,
/// }
/// ```
#[proc_macro_derive(Datex, attributes(datex))]
pub fn datex_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    datex_proxy::derive(input).into()
}

#[proc_macro_attribute]
pub fn datex(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let item = parse_macro_input!(input_clone as Item);
    generate_impl_glue_code(input.into(), item).into()
}
