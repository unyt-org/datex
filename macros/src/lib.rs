use proc_macro::TokenStream;
use syn::{ImplItemFn, ItemFn, ItemImpl, parse_macro_input};

use crate::test::create_async_test;
mod bitfield_macros;
mod lib_types;
mod test;
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

#[proc_macro_derive(LibTypeString)]
pub fn derive_lib_type_string(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    lib_types::derive_lib_type_string(input).into()
}

#[proc_macro_attribute]
pub fn async_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_function = parse_macro_input!(item as ItemFn);
    create_async_test(original_function).into()
}
