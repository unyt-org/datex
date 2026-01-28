use std::path::PathBuf;

use proc_macro2::TokenStream;

use std::{env, fs, str::FromStr};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    FnArg, Ident, ItemFn, LitStr, Pat, PatIdent, Signature, Token,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Type,
};

#[derive(Debug)]
pub struct ParsedAttributes {
    pub config: Option<PathBuf>,
}

// fn get_file_path() -> PathBuf {
//     let root_path = PathBuf::from_str(
//         &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()),
//     )
//     .unwrap();
//     root_path
//         .join(Span::call_site().file())
//         .canonicalize()
//         .unwrap()
// }

// impl Parse for ParsedAttributes {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let mut config = None;

//         let source_file = get_file_path();

//         // first try if directly a path string
//         if let Ok(config_path) = get_config_path(&input, &source_file) {
//             return Ok(ParsedAttributes {
//                 config: Some(config_path),
//             });
//         }

//         while !input.is_empty() {
//             let ident: Ident = input.parse()?;
//             input.parse::<Token![=]>()?;

//             if ident == "config" {
//                 config = Some(get_config_path(&input, &source_file)?);
//             } else {
//                 return Err(input.error("Unknown attribute"));
//             }

//             // optionally parse comma
//             if input.peek(Token![,]) {
//                 input.parse::<Token![,]>()?;
//             }
//         }

//         Ok(ParsedAttributes { config })
//     }
// }

// fn get_config_path(
//     input: &ParseStream,
//     source_file: &PathBuf,
// ) -> Result<PathBuf, syn::Error> {
//     if input.peek(LitStr) {
//         if let syn::Lit::Str(litstr) = input.parse()? {
//             let config_path_str = litstr.value();
//             let path = source_file
//                 .parent()
//                 .unwrap()
//                 .join(config_path_str)
//                 .canonicalize();
//             if let Ok(path) = path {
//                 Ok(path)
//             } else {
//                 return Err(input.error(path.unwrap_err().to_string()));
//             }
//         } else {
//             return Err(input
//                 .error("Invalid value for `config` - must be a path string"));
//         }
//     } else {
//         return Err(input.error("Not a string"));
//     }
// }

pub fn datex_main_impl(attrs: ParsedAttributes, input: ItemFn) -> TokenStream {
    if !input.sig.asyncness.is_some() {
        return syn::Error::new_spanned(
            &input.sig.fn_token,
            "the function annotated with #[datex_main] must be async",
        )
        .to_compile_error()
        .into();
    }

    if input.sig.ident != "main" {
        return syn::Error::new_spanned(
            &input.sig.ident,
            "the function annotated with #[datex_main] must be named `main`",
        )
        .to_compile_error()
        .into();
    }

    let arg_ident = match input.sig.inputs.len() {
        1 => match input.sig.inputs.first().unwrap() {
            FnArg::Typed(pat_ty) => match &*pat_ty.pat {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                other => {
                    return syn::Error::new_spanned(
                        other,
                        "expected an identifier argument like `runtime: Runtime`",
                    )
                    .to_compile_error()
                    .into();
                }
            },
            FnArg::Receiver(rcv) => {
                return syn::Error::new_spanned(
                    rcv,
                    "#[datex_main] cannot be used on methods",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input.sig.inputs,
                "expected exactly one argument: `fn main(runtime: Runtime)`",
            )
            .to_compile_error()
            .into();
        }
    };

    let user_impl_name = syn::Ident::new(
        &format!("__datex_main_impl_{}", input.sig.ident),
        input.sig.ident.span(),
    );
    let original_return_type = &input.sig.output;

    let output = quote! {
        #input

        #[tokio::main]
        async fn main() -> #original_return_type {
            datex_core::runtime::RuntimeRunner::new_native(
                datex_core::runtime::RuntimeConfig::default(),
                datex_core::runtime::GlobalRuntimeContext::default(),
            ).run(async move |#arg_ident| {
                #user_impl_name(#arg_ident).await
            }).await
        }
    };

    output.into()
}

// fn get_datex_config(
//     path: &PathBuf,
// ) -> Result<datex_core::runtime::RuntimeConfig, DeserializationError> {
//     let deserializer = DatexDeserializer::from_dx_file(path.clone())?;
//     let config: RuntimeConfig = Deserialize::deserialize(deserializer)?;
//     Ok(config)
// }

// fn compile_datex_config(path: &PathBuf) -> Vec<u8> {
//     let config_content =
//         fs::read_to_string(path).expect("failed to read DATEX config file");
//     let (dxb, _) = compile_script(&config_content, CompileOptions::default())
//         .expect("failed to compile DATEX config file");
//     dxb
// }
