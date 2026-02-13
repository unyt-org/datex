use std::path::PathBuf;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::{fs, env, str::FromStr};
use syn::{FnArg, Ident, ItemFn, LitStr, Pat, PatIdent, Token, parse::{Parse, ParseStream}, Signature, Attribute};
use crate::compiler::{compile_script, CompileOptions};
use crate::runtime::RuntimeConfig;
use crate::serde::deserializer::{from_dx_file, DatexDeserializer};
use crate::serde::error::DeserializationError;

#[derive(Debug)]
pub struct ParsedAttributes {
    pub config: Option<PathBuf>,
}

fn get_file_path() -> PathBuf {
    let root_path = PathBuf::from_str(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()),
    )
        .unwrap();
    root_path
        .join(Span::call_site().file())
        .canonicalize()
        .unwrap()
}

impl Parse for ParsedAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = None;

        let source_file = get_file_path();

        // first try if directly a path string
        if let Ok(config_path) = get_config_path(&input, &source_file) {
            return Ok(ParsedAttributes {
                config: Some(config_path),
            });
        }

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if ident == "config" {
                config = Some(get_config_path(&input, &source_file)?);
            } else {
                return Err(input.error("Unknown attribute"));
            }

            // optionally parse comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ParsedAttributes { config })
    }
}

fn get_config_path(
    input: &ParseStream,
    source_file: &PathBuf,
) -> Result<PathBuf, syn::Error> {
    if input.peek(LitStr) {
        if let syn::Lit::Str(litstr) = input.parse()? {
            let config_path_str = litstr.value();
            let path = source_file
                .parent()
                .unwrap()
                .join(config_path_str)
                .canonicalize();
            if let Ok(path) = path {
                Ok(path)
            } else {
                Err(input.error(path.unwrap_err().to_string()))
            }
        } else {
            Err(input
                .error("Invalid value for `config` - must be a path string"))
        }
    } else {
        Err(input.error("Not a string"))
    }
}

pub struct DatexMainInput<'a> {
    /// attributes of the main macro, e.g. config path
    pub parsed_attributes: ParsedAttributes,
    /// the function annotated with the macro, containing the application logic
    pub func: ItemFn,
    /// custom namespace for datex_core
    pub datex_core_namespace: &'a str,
    /// optional setup code to run before creating the runtime, e.g. for setting environment variables
    pub setup: Option<TokenStream>,
    /// optional initialization code to run after creating the runtime but before running the main function body
    pub init: Option<TokenStream>,
    /// additional attributes to add to the generated main function
    pub additional_attributes: Vec<Attribute>,
    /// custom input arguments for the main function, e.g. for providing additional dependencies
    pub custom_main_inputs: Vec<FnArg>,
    /// whether to enforce that the main function is named `main`
    pub enforce_main_name: bool,
}

/// Main implementation function for the datex_main macro
///
pub fn datex_main_impl(input: DatexMainInput) -> TokenStream {
    if input.func.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input.func.sig.fn_token,
            "the function must be async",
        )
            .to_compile_error();
    }

    if input.enforce_main_name && input.func.sig.ident != "main" {
        return syn::Error::new_spanned(
            &input.func.sig.ident,
            "the function must be named `main`",
        )
            .to_compile_error();
    }

    let arg_ident = match get_arg_ident(0, &input.func, "expected an identifier argument like `runtime: Runtime`") {
        Ok(ident) => ident,
        Err(err) => return err.to_compile_error(),
    };

    let ItemFn {
        mut sig,
        vis,
        block: body,
        attrs,
    } = input.func;

    sig.inputs.clear();
    for input in input.custom_main_inputs {
        sig.inputs.push(input);
    }

    let config_bytes = get_config_compiled_token_stream(&input.parsed_attributes);
    let core_namespace = syn::parse_str::<syn::Path>(input.datex_core_namespace).expect("invalid datex_core namespace");

    let additional_attributes = input.additional_attributes;
    let setup = input.setup;
    let init = input.init;

    let output = quote! {
        #(#additional_attributes)*
        #(#attrs)*
        #vis #sig {
            use #core_namespace::{runtime::{RuntimeRunner, RuntimeConfig}, serde::deserializer};

            {#setup}

            let config = match (#config_bytes) {
                Some(bytes) => deserializer::from_bytes(bytes).unwrap(),
                None => RuntimeConfig::default(),
            };

            let runner = RuntimeRunner::new(config);
            {
                let runtime = runner.runtime.clone();
                #init
            }
            runner.run(async move |#arg_ident| {
                #body
            }).await
        }
    };

    output
}

/// Helper function to get the compiled config as a byte array token stream, or None if no config path was provided
pub fn get_config_compiled_token_stream(parsed_attr: &ParsedAttributes) -> TokenStream {
    // try to get config from config path
    let config = parsed_attr.config.as_ref()
        .map(|path| get_datex_config(path).expect("failed to parse DATEX config file"));
    let config_bytes = parsed_attr.config.as_ref()
        .map(|path| compile_datex_config(path));

    config_bytes
        .map(|bytes| quote! {
            Some(&[#(#bytes),*])
        })
        .unwrap_or_else(|| quote! { None })
}


/// Helper function to get the identifier of the argument at the given index, or return a syn::Error if it's not an identifier or if it's a receiver (self)
pub fn get_arg_ident(index: usize, func: &ItemFn, err_msg: &'static str) -> Result<Ident, syn::Error> {
    match func.sig.inputs.get(index).unwrap() {
        FnArg::Typed(pat_ty) => match &*pat_ty.pat {
            Pat::Ident(PatIdent { ident, .. }) => Ok(ident.clone()),
            other => {
                Err(syn::Error::new_spanned(
                    other,
                    err_msg
                ))
            }
        },
        FnArg::Receiver(rcv) => {
            Err(syn::Error::new_spanned(
                rcv,
                "Expected typed argument, not self"
            ))
        }
    }
}


fn get_datex_config(path: &PathBuf) -> Result<RuntimeConfig, DeserializationError> {
    let config: RuntimeConfig = from_dx_file(path.clone())?;
    Ok(config)
}

fn compile_datex_config(path: &PathBuf) -> Vec<u8> {
    let config_content = fs::read_to_string(path).expect("failed to read DATEX config file");
    let (dxb, _) = compile_script(&config_content, CompileOptions::default()).expect("failed to compile DATEX config file");
    dxb
}