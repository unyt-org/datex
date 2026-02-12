use datex_core::{
    compiler::{CompileOptions, compile_template},
    values::value_container::ValueContainer,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, LitStr, Result,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

use crate::utils::expr_to_value_container;

pub struct PrecompileInput {
    pub script: String,
    pub args: Vec<ValueContainer>,
}

impl Parse for PrecompileInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let pattern: LitStr = input.parse()?;
        let _ = input.parse::<Option<Comma>>()?;
        let args = Punctuated::<Expr, Comma>::parse_terminated(input)?
            .iter()
            .map(expr_to_value_container)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            script: pattern.value(),
            args,
        })
    }
}

pub fn precompile(input: PrecompileInput) -> TokenStream {
    let PrecompileInput { script, args, .. } = input;
    let dxb = compile_template(
        &script,
        &args.iter().cloned().map(Some).collect::<Vec<_>>(),
        CompileOptions::default(),
    )
    .unwrap()
    .0;
    quote! {
        vec![#(#dxb),*]
    }
}
