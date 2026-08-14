use crate::utils::{get_datex_core_crate_name, get_project_relative_file_path};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Item, fold::{self, Fold}, Type};


pub fn generate_impl_glue_code(input: TokenStream, item: Item) -> TokenStream {
    let datex_core_crate_name = get_datex_core_crate_name();

    match &item {
        Item::Impl(item_impl) => {
            let self_ty = &item_impl.self_ty;

            let mut methods = Vec::new();
            let mut static_methods = Vec::new();

            for impl_item in &item_impl.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    let method_ident = &method.sig.ident;
                    let method_name = method_ident.to_string();
                    let is_async = method.sig.asyncness.is_some();

                    let return_type = match &method.sig.output {
                        syn::ReturnType::Default => None,
                        syn::ReturnType::Type(_, box ty) if matches!(ty, syn::Type::Tuple(tuple) if tuple.elems.is_empty()) => None,
                        syn::ReturnType::Type(_, ty) => Some(ty),
                    };
                    // if return type is Result<T, E>, extract T and E for yeet_type and return_type
                    let (return_type, yeet_type) = extract_return_and_yeet_type(return_type);

                    let return_type = return_type.map(|ty| replace_self_type(&ty, &item_impl.self_ty));
                    let yeet_type = yeet_type.map(|ty| replace_self_type(&ty, &item_impl.self_ty));

                    let return_type_tokens = match return_type {
                        Some(ref ty) => quote! { Some(Box::new(#ty::datex_type(memory))) },
                        None => quote! { None },
                    };
                    let yeet_type_tokens = match yeet_type {
                        Some(ref ty) => quote! { Some(Box::new(#ty::datex_type(memory))) },
                        None => quote! { None },
                    };


                    let mut parameter_defs = Vec::new();
                    let mut is_method = false;

                    for param in &method.sig.inputs {
                        match param {
                            syn::FnArg::Receiver(_) => {
                                is_method = true;
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
                    
                    let has_mutable_inputs = method.sig.inputs.iter().any(|param| {
                        match param {
                            syn::FnArg::Receiver(receiver) => receiver.mutability.is_some(),
                            syn::FnArg::Typed(pat_type) => {
                                if let Type::Reference(type_reference) = &*pat_type.ty {
                                    type_reference.mutability.is_some()
                                } else {
                                    false
                                }
                            }
                        }
                    });

                    let mut call_argument_inits = Vec::new();
                    let mut call_arguments = Vec::new();
                    for (index, param) in method.sig.inputs.iter().enumerate() {
                        let ty = match param {
                            syn::FnArg::Receiver(receiver) => {
                                *item_impl.self_ty.clone()
                            }
                            syn::FnArg::Typed(pat_type) => {
                                replace_self_type(&pat_type.ty, &item_impl.self_ty)
                            }
                        };
                        let var_ident = syn::Ident::new(&format!("arg_{}", index), proc_macro2::Span::call_site());
                        call_argument_inits.push(
                            quote! {
                                let mut #var_ident = #ty::try_from_value_container(vals.pop().unwrap()).unwrap();
                            }
                        );
                        // distinguish between move, & and &mut
                        call_arguments.push(match param {
                            syn::FnArg::Receiver(receiver) => {
                                if receiver.reference.is_some() {
                                    if receiver.mutability.is_some() {
                                        quote! { &mut #var_ident }
                                    } else {
                                        quote! { &#var_ident }
                                    }
                                } else {
                                    quote! { #var_ident }
                                }
                            }
                            syn::FnArg::Typed(pat_type) => {
                                if let syn::Type::Reference(type_reference) = &*pat_type.ty {
                                    if type_reference.mutability.is_some() {
                                        quote! { &mut #var_ident }
                                    } else {
                                        quote! { &#var_ident }
                                    }
                                } else {
                                    quote! { #var_ident }
                                }
                            }
                        });
                    }

                    // reverse the call_arguments initialization, but keep usage in the original order
                    call_argument_inits.reverse();

                    let method_call_body = quote! {{
                        #(#call_argument_inits)*
                        #self_ty::#method_ident(#(#call_arguments),*)
                    }};

                    // with return type, wrap in Some, otherwise return None
                    let method_call = if return_type.is_some() {
                        quote! {
                            Some(ValueContainer::try_from(#method_call_body).unwrap())
                        }
                    } else {
                        quote! {{
                            #method_call_body;
                            None
                        }}
                    };
                    
                    let kind = if has_mutable_inputs {
                        quote! { CallableKind::Procedure }
                    } else {
                        quote! { CallableKind::Function }
                    };

                    let callable = quote! {
                        Callable {
                            name: Some(#method_name.to_string()),
                            signature: CallableTypeDefinition {
                                kind: #kind,
                                requires_async: #is_async,
                                parameters: vec![#(#parameter_defs),*],
                                rest_parameter: None,
                                return_type: #return_type_tokens,
                                yeet_type: #yeet_type_tokens,
                            },
                            body: CallableBody::native_sync(|mut vals| {Ok(#method_call)}),
                            creator: Default::default(),
                        }
                    };

                    if is_method {
                        methods.push(quote! {
                            EntityImplMethod {
                                call_on_owner: true,
                                callable: #callable,
                            }
                        })
                    }
                    else {
                        static_methods.push(quote! {
                            #callable
                        })
                    }
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
                                static_methods: vec![#(#static_methods),*],
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

fn extract_return_and_yeet_type(
    return_type: Option<&Box<Type>>,
) -> (Option<Type>, Option<Type>) {
    if let Some(ty) = return_type {
        if let syn::Type::Path(type_path) = &**ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Result" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        let mut args_iter = args.args.iter();
                        let ok_type = args_iter.next().and_then(|arg| {
                            if let syn::GenericArgument::Type(ty) = arg {
                                Some(ty)
                            } else {
                                None
                            }
                        });
                        let err_type = args_iter.next().and_then(|arg| {
                            if let syn::GenericArgument::Type(ty) = arg {
                                Some(ty)
                            } else {
                                None
                            }
                        });
                        (ok_type.cloned(), err_type.cloned())
                    } else {
                        (Some(*ty.clone()), None)
                    }
                } else {
                    (Some(*ty.clone()), None)
                }
            } else {
                (Some(*ty.clone()), None)
            }
        } else {
            (Some(*ty.clone()), None)
        }
    } else {
        (None, None)
    }
}

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