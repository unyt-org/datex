use datex_core::macro_utils::entrypoint::{
    DatexMainInput, ParsedAttributes, datex_main_impl,
};
use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input, parse_quote};

extern crate alloc;

use crate::{compile::PrecompileInput, execute::ExecuteMacroInput};

mod compile;
mod execute;
mod utils;

#[proc_macro]
pub fn precompile(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PrecompileInput);
    compile::precompile(input).into()
}

#[proc_macro]
pub fn execute_sync(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExecuteMacroInput);
    execute::execute_sync(input).into()
}

#[proc_macro]
pub fn execute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExecuteMacroInput);
    execute::execute_async(input).into()
}

#[proc_macro]
pub fn execute_sync_unchecked(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExecuteMacroInput);
    execute::execute_sync_unchecked(input).into()
}

#[proc_macro]
pub fn execute_unchecked(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExecuteMacroInput);
    execute::execute_async_unchecked(input).into()
}

/// The main entry point for a DATEX application, providing a DATEX runtime instance
#[proc_macro_attribute]
pub fn datex_main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_attributes = parse_macro_input!(attr as ParsedAttributes);

    let original_function = parse_macro_input!(item as ItemFn);
    datex_main_impl(DatexMainInput {
        parsed_attributes,
        func: original_function,
        datex_core_namespace: "datex_core",
        setup: None,
        init: None,
        pre_body: None,
        additional_attributes: vec![parse_quote! {#[tokio::main]}],
        custom_main_inputs: vec![],
        enforce_main_name: false,
    })
    .into()
}
