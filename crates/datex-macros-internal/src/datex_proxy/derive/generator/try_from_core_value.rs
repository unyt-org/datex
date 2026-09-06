use crate::datex_proxy::data::StructureData;
use proc_macro2::TokenStream;
use quote::quote;

/// Generates the from core value traits
pub fn generate_try_from_core_value(
    structure_data: &StructureData,
) -> TokenStream {
    let StructureData {
        ident, generics, ..
    } = structure_data;

    quote! {
        #[automatically_derived]
        impl #generics ConvertCoreValue for #ident #generics {
            fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue> {
                match value {
                    CoreValue::Native(native) => native.try_into_value().map_err(CoreValue::Native),
                    _ => Err(value),
                }
            }

            fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()> { // FIXME: generics
                match value {
                    CoreValue::Native(native) => native.try_as().ok_or(()),
                    _ => Err(()),
                }
            }

            fn try_borrow_mut_from_core_value(value: &mut CoreValue) -> Result<&mut Self, ()> {
                match value {
                    CoreValue::Native(native) => native.try_as_mut().ok_or(()),
                    _ => Err(()),
                }
            }
        }

        // FIXME: cannot implement for Goat (foreign trait)
        //
        // #[automatically_derived]
        // impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, #ident> {
        //     type Error = ();
        //     fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        //         match value {
        //             BorrowedCoreValue::Native(native) => native.filter_map(|v| v.as_any().downcast_ref::<#ident>()).ok_or(()),
        //             _ => Err(()),
        //         }
        //     }
        // }
        //
        // #[automatically_derived]
        // impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, #ident> {
        //     type Error = ();
        //     fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        //         match value {
        //             BorrowedCoreValueMut::Native(native) => native.filter_map(|v| v.as_any_mut().downcast_mut::<#ident>()).ok_or(()),
        //             _ => Err(()),
        //         }
        //     }
        // }
    }
}
