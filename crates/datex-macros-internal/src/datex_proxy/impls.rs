use crate::{
    datex_proxy::native_callable::generate_native_callable_from_impl_fn,
    utils::{get_datex_core_crate_name, get_project_relative_file_path},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Item, ItemImpl};

pub fn generate_impl_glue_code(
    input: TokenStream,
    item: &ItemImpl,
) -> TokenStream {
    let datex_core_crate_name = get_datex_core_crate_name();
    let self_ty = &item.self_ty;

    let mut methods = Vec::new();
    let mut static_methods = Vec::new();

    for impl_item in &item.items {
        if let syn::ImplItem::Fn(method) = impl_item {
            let is_method = method
                .sig
                .inputs
                .first()
                .map(|arg| matches!(arg, syn::FnArg::Receiver(_)))
                .unwrap_or(false);
            let callable =
                generate_native_callable_from_impl_fn(method, self_ty);

            if is_method {
                methods.push(quote! {
                    EntityImplMethod {
                        call_on_owner: true,
                        callable: #callable,
                    }
                })
            } else {
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
    let name = item.self_ty.to_token_stream().to_string();

    quote! {
        #input

        const _: () = {
            use #datex_core_crate_name::{
                prelude::*,
                types::type_definition::callable::{CallableKind, CallableTypeDefinition},
                values::core_values::callable::{Callable, CallableBody, NativeCallable},
                types::entities::entity_impls::EntityImplMethod,
                values::core_values::callable::{native_sync_callable, native_async_callable},
                datex_proxy::DatexValueContainerProxyDeserialize,
                values::value_container::ValueContainer,
            };

            #datex_core_crate_name::inventory::submit! {
                #datex_core_crate_name::datex_registry::DatexImplRegistration {
                    namespace: #namespace,
                    name: #name,
                    create_impl: |cache| #datex_core_crate_name::types::entities::entity_impls::EntityImpl {
                        methods: vec![#(#methods),*],
                        static_methods: vec![#(#static_methods),*],
                    },
                    owner_type_id: || { ::core::any::TypeId::of::<#self_ty>()},
                }
            }
        };
    }
}
