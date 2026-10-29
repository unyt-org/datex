use proc_macro2::TokenStream;
use quote::quote;
use crate::datex_proxy::data::StructureData;

/// Generates a TokenStream that creates a PointerAddress for the given structure data.
pub fn get_pointer_address_for_struct(
    structure_data: &StructureData,
) -> TokenStream {
    // builtin address
    if let Some(builtin_address) = &structure_data.attributes.builtin_address {
        quote! {
            PointerAddress::Remote(RemotePointerAddress::builtin_u32(#builtin_address))
        }
    }

    else {
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