use std::str::FromStr;

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

pub struct PrecompileInput {
    pub script: String,
    pub args: Vec<ValueContainer>,
}

impl Parse for PrecompileInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let pattern: LitStr = input.parse()?;
        let _ = input.parse::<Option<Comma>>()?;
        let args = Punctuated::<Expr, Comma>::parse_terminated(input)?.iter().map(|exp| match exp {
            Expr::Lit(lit) => match &lit.lit {
                syn::Lit::Str(s) => ValueContainer::from(s.value()),
                syn::Lit::Int(i) => {
                    let variant =  if i.suffix().is_empty() {
                        IntegerTypeVariant::I32
                    } else {
                        IntegerTypeVariant::from_str(i.suffix()).unwrap()
                    };
                    ValueContainer::from(TypedInteger::from_string_with_variant(i.base10_digits(), variant).unwrap())
                },
                syn::Lit::Float(f) => {
                    let suffix = if f.suffix().is_empty() {
                        "f64"
                    } else {
                        f.suffix()
                    };
                    match suffix {
                        "f32" => ValueContainer::from(f.base10_parse::<f32>().unwrap()),
                        "f64" => ValueContainer::from(f.base10_parse::<f64>().unwrap()),
                        _ => panic!("Unsupported float literal suffix: {}", suffix),
                    }
                }
                syn::Lit::Bool(b) => ValueContainer::from(b.value),
                _ => panic!("Only string and integer literals are supported as arguments"),
            },
            _ => panic!("Only literal expressions are supported as arguments"),
        }).collect::<Vec<_>>();
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
