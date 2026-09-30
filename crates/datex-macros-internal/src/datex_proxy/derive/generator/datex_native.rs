use proc_macro2::TokenStream;
use quote::quote;

use crate::datex_proxy::data::{StructureData, TypeKind};

/// Generates the [DatexNative] implementation, including [AsBorrowed] and [AsBorrowedMut] implementations for the given structure data.
/// Returns a TokenStream of the implementations.
pub fn generate_datex_native(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    let native_only_structural_impl =
        generate_datex_native_only_structural(structure_data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        use core::any::Any;

        impl #impl_generics DatexNative for #ident #ty_generics #where_clause {
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
        }

        // TODO move to separate mods
        impl #impl_generics DatexNativeOps for #ident #ty_generics #where_clause {}
        impl #impl_generics LocalChildPathResolver for #ident #ty_generics #where_clause {}
        impl #impl_generics UpdateHandlerImpl for #ident #ty_generics #where_clause {}
        impl #impl_generics UpdateCallbackDataAccess for #ident #ty_generics #where_clause {}

        #native_only_structural_impl
    }
}

pub fn generate_datex_native_only_structural(
    structure_data: &StructureData,
) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // TODO: validate that all children also implement DatexNativeOnlyStructural
    match structure_data.attributes.type_kind {
        TypeKind::Structural {
            only_structural: false,
        } => quote! {
            impl #impl_generics DatexNativeStructural for #ident #ty_generics #where_clause {}
        },
        TypeKind::Structural {
            only_structural: true,
        } => quote! {
            impl #impl_generics DatexNativeStructural for #ident #ty_generics #where_clause {}
            impl #impl_generics DatexNativeOnlyStructural for #ident #ty_generics #where_clause {}
        },
        _ => quote! {},
    }
}
