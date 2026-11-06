use crate::datex_proxy::data::StructureData;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Expr, Lit, Meta};

/// Generates a TokenStream that creates a PointerAddress for the given structure data.
pub fn get_pointer_address_for_struct(
    structure_data: &StructureData,
) -> TokenStream {
    // builtin address
    if let Some(builtin_address) = &structure_data.attributes.builtin_address {
        quote! {
            PointerAddress::Remote(RemotePointerAddress::builtin_u32(#builtin_address))
        }
    } else {
        // FIXME: calculate pointer address statically in macro at compile time
        let unique_name = format!(
            "{}::{}",
            structure_data.namespace.join("::"),
            structure_data.ident
        );
        quote! {
            unsafe {
                PointerAddress::SelfOwned(SelfOwnedPointerAddress::new_static_from_name(
                    #unique_name
                ))
            }
        }
    }
}

/// Generates a TokenStream that creates a SelfOwnedPointerAddress for the given function.
pub fn get_pointer_address_for_fn(
    namespace: &str,
    fn_name: &str,
) -> TokenStream {
    let unique_name = format!("{}::{}", namespace, fn_name);
    quote! {
        unsafe {
            SelfOwnedPointerAddress::new_static_from_name(
                #unique_name
            )
        }
    }
}

pub fn parse_doc_comments(attrs: &[Attribute]) -> Option<String> {
    let docs = attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }
            let Meta::NameValue(name_value) = &attr.meta else {
                return None;
            };
            let Expr::Lit(expr_lit) = &name_value.value else {
                return None;
            };
            let Lit::Str(lit_str) = &expr_lit.lit else {
                return None;
            };
            Some(lit_str.value().trim_start().to_string())
        })
        .collect::<Vec<_>>();
    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}
