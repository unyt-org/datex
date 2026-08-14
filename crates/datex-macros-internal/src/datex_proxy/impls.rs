use crate::utils::{get_datex_core_crate_name, get_project_relative_file_path};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Item,
    fold::{self, Fold},
};

/// A helper struct to replace occurrences of `Self` in a type with a specific type.
/// TODO: Move to utils
struct SelfTypeReplacer<'a> {
    self_ty: &'a syn::Type,
}

impl Fold for SelfTypeReplacer<'_> {
    fn fold_type(&mut self, ty: syn::Type) -> syn::Type {
        match &ty {
            syn::Type::Path(path)
                if path.qself.is_none() && path.path.is_ident("Self") =>
            {
                self.self_ty.clone()
            }
            _ => fold::fold_type(self, ty),
        }
    }
}
/// A helper function to replace occurrences of `Self` in a type with a specific type.
fn replace_self_type(ty: &syn::Type, self_ty: &syn::Type) -> syn::Type {
    SelfTypeReplacer { self_ty }.fold_type(ty.clone())
}

pub fn generate_impl_glue_code(input: TokenStream, item: Item) -> TokenStream {
    let datex_core_crate_name = get_datex_core_crate_name();

    match &item {
        Item::Impl(item_impl) => {
            let self_ty = &item_impl.self_ty;

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
                                let name = match &*pat_type.pat {
                                    syn::Pat::Ident(ident) => {
                                        ident.ident.to_string()
                                    }
                                    _ => {
                                        panic!("Unsupported parameter pattern")
                                    }
                                };
                                // normalize type name (replace Self with the actual type name)
                                let ty =
                                    replace_self_type(&pat_type.ty, self_ty);
                                parameter_defs.push(quote! {
                                    (
                                        Some(#name.to_string()),
                                        <#ty as DatexProxyTypes>::datex_type(cache)
                                    )
                                });
                            }
                        }
                    }
                    let return_type = match &method.sig.output {
                        syn::ReturnType::Default => {
                            quote!(None)
                        }
                        syn::ReturnType::Type(_, box ty) if matches!(ty, syn::Type::Tuple(tuple) if tuple.elems.is_empty()) =>
                        {
                            quote!(None)
                        }
                        syn::ReturnType::Type(_, ty) => {
                            let ty = replace_self_type(ty, self_ty);
                            quote! {
                                Some(Box::new(<#ty as DatexProxyTypes>::datex_type(cache)))
                            }
                        }
                    };
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
                                    return_type: #return_type,
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
                            create_impl: |cache| {
                                #datex_core_crate_name::types::entities::entity_impls::EntityImpl {
                                    methods: vec![
                                        #(#methods),*
                                    ],
                                    static_methods: vec![],
                                }
                            },
                            owner_type_id: || { ::core::any::TypeId::of::<#self_ty>()},
                        }
                    }
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
