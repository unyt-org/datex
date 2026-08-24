use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Item, ItemMod, LitStr};
use syn::parse::Parse;
use crate::datex_proxy::native_callable::generate_native_callable_from_fn;
use crate::utils::get_datex_core_crate_name;

struct ModuleAttributes {
    name: Option<String>,
}

impl Parse for ModuleAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name: Option<LitStr> = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;

            if ident == "name" {
                name = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "unknown datex attribute",
                ));
            }

            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self {
            name: name.map(|lit| lit.value())
        })
    }
}


pub fn generate_mod_glue_code(
    args: Option<TokenStream>,
    item: &ItemMod,
) -> TokenStream {
    let items = if let Some((_, items)) = &item.content {
        items
    } else {
        panic!("The #[datex] attribute can only be applied to inline modules.");
    };

    let datex_core_crate_name = get_datex_core_crate_name();
    let attrs = &item.attrs;
    let module_attributes: Option<ModuleAttributes> = args.map(|args| syn::parse2(args).unwrap());

    // datex module name either #[datex(name = "custom_name")] or the module name itself
    let datex_module_name = module_attributes
        .and_then(|attrs| attrs.name)
        .unwrap_or_else(|| item.ident.to_string());

    let vis = &item.vis;
    let ident = &item.ident;
    
    // map items to their token streams
    let datex_items: Vec<TokenStream> = items.iter().filter_map(|item| map_to_datex_item(item)).collect();

    quote! {
        #(#attrs)*
        #vis mod #ident {
            #(#items)*
            
            #datex_core_crate_name::inventory::submit! {
                #datex_core_crate_name::datex_registry::DatexModuleRegistration {
                    name: #datex_module_name,
                    create_module: |cache| #datex_core_crate_name::values::core_values::map::Map::structural_with_string_keys(
                        vec![#(#datex_items),*],
                    ),
                }
            }
        }
    }
}

fn map_to_datex_item(item: &Item) -> Option<TokenStream> {
    match item {
        // only register pub functions
        Item::Fn(item_fn) if item_fn.vis.to_token_stream().to_string().contains("pub") => {
            let mapped = generate_native_callable_from_fn(
                item_fn,
            );
            let name = item_fn.sig.ident.to_string();

            Some(
                quote! {
                    (
                        #name.to_string(),
                        #mapped.into(),
                    )
                }
            )
        }
        _ => {
           None
        }
    }
}