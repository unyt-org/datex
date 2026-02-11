use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::{compile::PrecompileInput, execute::ExecuteMacroInput};
mod compile;
mod execute;

#[proc_macro]
pub fn precompile(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PrecompileInput);
    compile::precompile(input).into()
}

#[proc_macro]
pub fn execute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExecuteMacroInput);
    execute::execute(input).into()
}
