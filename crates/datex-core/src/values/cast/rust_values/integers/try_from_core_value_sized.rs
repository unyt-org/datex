use crate::traits::convert_core_value::ConvertCoreValue;
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::core_value::CoreValue;
use crate::values::core_values::integer::typed_integer::TypedInteger;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut};

macro_rules! impl_pointer_sized_core_value_conversions {
    ($($ty:ident => $variant:ident, $repr:ty, $borrow:ident, $borrow_mut:ident;)* $(,)?) => {
        $(
            const _: () = {
                assert!(core::mem::size_of::<$ty>() == core::mem::size_of::<$repr>());
                assert!(core::mem::align_of::<$ty>() == core::mem::align_of::<$repr>());
            };

            impl ConvertCoreValue for $ty {
                fn try_from_core_value(value: CoreValue) -> Result<Self, ()> {
                    match value {
                        CoreValue::TypedInteger(TypedInteger::$variant(v)) => Ok(v as $ty),
                        CoreValue::Native(native) => native.try_into_value().ok_or(()),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<&'a CoreValue> for &'a $ty {
                type Error = ();
                fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        // SAFETY: checked above to have identical size and alignment,
                        // and both are plain integers with no padding or niches.
                        CoreValue::TypedInteger(TypedInteger::$variant(v)) => {
                            Ok(unsafe { &*(v as *const $repr as *const $ty) })
                        }
                        CoreValue::Native(native) => native.try_as().ok_or(()),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<&'a mut CoreValue> for &'a mut $ty {
                type Error = ();
                fn try_from(value: &'a mut CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        // SAFETY: as above; every bit pattern is valid for both types,
                        // so writes through the resulting reference are also fine.
                        CoreValue::TypedInteger(TypedInteger::$variant(v)) => {
                            Ok(unsafe { &mut *(v as *mut $repr as *mut $ty) })
                        }
                        CoreValue::Native(native) => native.try_as_mut().ok_or(()),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, $ty> {
                type Error = ();
                fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
                    match value {
                        BorrowedCoreValue::TypedInteger(v) => v
                            // SAFETY: see above.
                            .filter_map(|v| {
                                v.$borrow().map(|v| unsafe { &*(v as *const $repr as *const $ty) })
                            })
                            .ok_or(()),
                        BorrowedCoreValue::Native(native) => native
                            .filter_map(|v| v.as_any().downcast_ref::<$ty>())
                            .ok_or(()),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, $ty> {
                type Error = ();
                fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
                    match value {
                        BorrowedCoreValueMut::TypedInteger(v) => v
                            // SAFETY: see above.
                            .filter_map(|v| {
                                v.$borrow_mut().map(|v| unsafe { &mut *(v as *mut $repr as *mut $ty) })
                            })
                            .ok_or(()),
                        BorrowedCoreValueMut::Native(native) => native
                            .filter_map(|v| v.as_any_mut().downcast_mut::<$ty>())
                            .ok_or(()),
                        _ => Err(()),
                    }
                }
            }
        )*
    };
}

#[cfg(target_pointer_width = "64")]
impl_pointer_sized_core_value_conversions! {
    usize => U64, u64, borrow_as_u64, borrow_mut_as_u64;
    isize => I64, i64, borrow_as_i64, borrow_mut_as_i64;
}

#[cfg(target_pointer_width = "32")]
impl_pointer_sized_core_value_conversions! {
    usize => U32, u32, borrow_as_u32, borrow_mut_as_u32;
    isize => I32, i32, borrow_as_i32, borrow_mut_as_i32;
}

#[cfg(target_pointer_width = "16")]
impl_pointer_sized_core_value_conversions! {
    usize => U16, u16, borrow_as_u16, borrow_mut_as_u16;
    isize => I16, i16, borrow_as_i16, borrow_mut_as_i16;
}