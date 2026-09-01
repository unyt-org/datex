use proc_macro2::TokenStream;
use quote::quote;
use crate::datex_proxy::data::StructureData;

/// Generates the from core value traits
pub fn generate_try_from_core_value(structure_data: &StructureData) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    quote! {
        #[automatically_derived]
        impl #generics TryFrom<CoreValue> for #ident #generics {
            type Error = ();
            fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
                match value {
                    CoreValue::Native(native) => native.try_into_value().ok_or(()),
                    _ => Err(()),
                }
            }
        }
        
        #[automatically_derived]
        impl<'a> TryFrom<&'a CoreValue> for &'a #ident { // FIXME: generics
            type Error = ();
            fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
                match value {
                    CoreValue::Native(native) => native.try_as().ok_or(()),
                    _ => Err(()),
                }
            }
        }

        #[automatically_derived]
        impl<'a> TryFrom<&'a mut CoreValue> for &'a mut #ident {
            type Error = ();
            fn try_from(value: &'a mut CoreValue) -> Result<Self, Self::Error> {
                match value {
                    CoreValue::Native(native) => native.try_as_mut().ok_or(()),
                    _ => Err(()),
                }
            }
        }

        #[automatically_derived]
        impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, #ident> {
            type Error = ();
            fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
                match value {
                    BorrowedCoreValue::Native(native) => native.filter_map(|v| v.as_any().downcast_ref::<#ident>()).ok_or(()),
                    _ => Err(()),
                }
            }
        }

        #[automatically_derived]
        impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, #ident> {
            type Error = ();
            fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
                match value {
                    BorrowedCoreValueMut::Native(native) => native.filter_map(|v| v.as_any_mut().downcast_mut::<#ident>()).ok_or(()),
                    _ => Err(()),
                }
            }
        }
    }
}