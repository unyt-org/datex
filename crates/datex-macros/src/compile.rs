use core::str::FromStr;

use datex_core::{
    compiler::{CompileOptions, compile_template},
    values::{
        core_values::integer::typed_integer::{
            IntegerTypeVariant, TypedInteger,
        },
        value_container::ValueContainer,
    },
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
            .collect::<Vec<_>>();
        Ok(Self {
            script: pattern.value(),
            args,
        })
    }
}

pub fn precompile(input: PrecompileInput) -> TokenStream {
    let PrecompileInput { script, args, .. } = input;
    let dxb = compile_template(&script, &args, CompileOptions::default())
        .unwrap()
        .0;
    quote! {
        vec![#(#dxb),*]
    }
}
