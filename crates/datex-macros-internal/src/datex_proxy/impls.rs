use crate::utils::{get_datex_core_crate_name, get_project_relative_file_path};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Item;

pub fn generate_impl_glue_code(input: TokenStream, item: Item) -> TokenStream {
    let datex_core_crate_name = get_datex_core_crate_name();

    match &item {
        Item::Impl(item_impl) => {
            let mut methods = Vec::new();

            for impl_item in &item_impl.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    let name = method.sig.ident.to_string();

                    let mut parameter_defs = Vec::new();
                    for param in &method.sig.inputs {
                        match param {
                            syn::FnArg::Receiver(_) => {
                                // todo
                            }
                            syn::FnArg::Typed(pat_type) => {
                                let ty = &pat_type.ty;
                                let name = match &*pat_type.pat {
                                    syn::Pat::Ident(ident) => {
                                        ident.ident.to_string()
                                    }
                                    _ => {
                                        panic!("Unsupported parameter pattern")
                                    }
                                };
                                parameter_defs.push(quote!{
                                    (Some(#name.to_string()), #ty::datex_type(memory))
                                });
                            }
                        }
                    }

                    methods.push(quote! {
                        EntityImplMethod {
                            call_on_owner: true,
                            callable: Callable {
                                name: Some(#name.to_string()),
                                signature: CallableTypeDefinition {
                                    kind: CallableKind::Procedure,
                                    requires_async: false,
                                    parameters: vec![#(#parameter_defs),*],
                                    rest_parameter: None,
                                    return_type: None,
                                    yeet_type: None,
                                },
                                body: CallableBody::native_sync(|vals| {Ok(None)}),
                                creator: Default::default(),
                            }
                        }

                    })
                }
            }

            let namespace = {
                let mut ns = get_project_relative_file_path();
                ns.set_extension("");
                ns.to_str()
                    .expect("Failed to convert file path to string")
                    .to_string()
            };
            let name = item_impl.self_ty.to_token_stream().to_string();

            quote! {
                #input

                const _: () = {
                    use #datex_core_crate_name::{
                        prelude::*,
                        types::type_definition::callable::{CallableKind, CallableTypeDefinition},
                        values::core_values::callable::{Callable, CallableBody, NativeCallable},
                        types::entities::entity_impls::EntityImplMethod,
                    };

                    #datex_core_crate_name::inventory::submit! {
                        #datex_core_crate_name::datex_registry::DatexImplRegistration {
                            namespace: #namespace,
                            name: #name,
                            create_impl: |memory| #datex_core_crate_name::types::entities::entity_impls::EntityImpl {
                                methods: vec![#(#methods),*],
                                static_methods: vec![],
                            }
                        }
                    };
                };
            }
        }
        _ => {
            panic!(
                "The #[datex_proxy] attribute can only be applied to impl blocks."
            );
        }
    }
}
