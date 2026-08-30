use proc_macro2::TokenStream;
use quote::quote;

use crate::datex_proxy::data::{StructureData, TypeKind};

/// Generates the [DatexNative] implementation, including [AsBorrowed] and [AsBorrowedMut] implementations for the given structure data.
/// Returns a TokenStream of the implementations.
pub fn generate_datex_native(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;
    
    let native_only_structural_impl = generate_datex_native_only_structural(structure_data);
    
    quote! {
        use core::any::Any;
        
        impl #generics DatexNative for #ident #generics {
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
            fn value_datex_type(&self, cache: &mut SharedReferencesCache) -> Type {
                Self::datex_type(cache)
            }
        }
        
        #native_only_structural_impl

        #[automatically_derived]
        impl<'a> AsBorrowed<'a> for #ident #generics { // FIXME: generics
            fn as_borrowed(&'a self) -> BorrowedValueContainer<'a> {
                BorrowedValueContainer::native_borrowed(self)
            }
        }

        #[automatically_derived]
        impl<'a> AsBorrowedMut<'a> for #ident #generics { // FIXME: generics
            fn as_borrowed_mut(&'a mut self) -> BorrowedValueContainerMut<'a> {
                BorrowedValueContainerMut::native_borrowed(self)
            }
        }
    }
}

pub fn generate_datex_native_only_structural(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;
    // TODO: validate that all children also implement DatexNativeOnlyStructural
    match structure_data.attributes.type_kind {
        TypeKind::Structural { only_structural: true } => quote! {
            impl #generics DatexNativeOnlyStructural for #ident #generics {}
        },
        _ => quote! {}
    }
}