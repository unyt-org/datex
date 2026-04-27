use proc_macro::TokenStream;
use syn::parse_macro_input;

mod bitfield_macros;
mod core_lib;
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
