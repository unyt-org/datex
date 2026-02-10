use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use crate::compile::PrecompileInput;
mod compile;

#[proc_macro]
pub fn precompile(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PrecompileInput);
    compile::precompile(input).into()
}
