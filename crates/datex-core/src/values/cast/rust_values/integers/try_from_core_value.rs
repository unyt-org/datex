use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::core_value::CoreValue;
use crate::values::core_values::integer::typed_integer::TypedInteger;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut};

macro_rules! impl_integer_core_value_conversions {
    ($($ty:ident => $variant:ident, $borrow:ident, $borrow_mut:ident;)* $(,)?) => {
        $(
            impl TryFrom<CoreValue> for $ty {
                type Error = ();
                fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        CoreValue::TypedInteger(TypedInteger::$variant(v)) => Ok(v),
                        CoreValue::Native(native) => native.try_into_value().ok_or(()),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<&'a CoreValue> for &'a $ty {
                type Error = ();
                fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        CoreValue::TypedInteger(TypedInteger::$variant(v)) => Ok(v),
                        CoreValue::Native(native) => native.try_as().ok_or(()),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<&'a mut CoreValue> for &'a mut $ty {
                type Error = ();
                fn try_from(value: &'a mut CoreValue) -> Result<Self, Self::Error> {
                    match value {
                        CoreValue::TypedInteger(TypedInteger::$variant(v)) => Ok(v),
                        CoreValue::Native(native) => native.try_as_mut().ok_or(()),
                        _ => Err(()),
                    }
                }
            }

            impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, $ty> {
                type Error = ();
                fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
                    match value {
                        BorrowedCoreValue::TypedInteger(v) => {
                            v.filter_map(|v| v.$borrow()).ok_or(())
                        }
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
                        BorrowedCoreValueMut::TypedInteger(v) => {
                            v.filter_map(|v| v.$borrow_mut()).ok_or(())
                        }
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

impl_integer_core_value_conversions! {
    u8   => U8,   borrow_as_u8,   borrow_mut_as_u8;
    u16  => U16,  borrow_as_u16,  borrow_mut_as_u16;
    u32  => U32,  borrow_as_u32,  borrow_mut_as_u32;
    u64  => U64,  borrow_as_u64,  borrow_mut_as_u64;
    u128 => U128, borrow_as_u128, borrow_mut_as_u128;
    i8   => I8,   borrow_as_i8,   borrow_mut_as_i8;
    i16  => I16,  borrow_as_i16,  borrow_mut_as_i16;
    i32  => I32,  borrow_as_i32,  borrow_mut_as_i32;
    i64  => I64,  borrow_as_i64,  borrow_mut_as_i64;
    i128 => I128, borrow_as_i128, borrow_mut_as_i128;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::core_value::CoreValue;
    use crate::values::core_values::boolean::Boolean;

    #[test]
    fn try_bool_from_core_value() {
        let mut core_value = CoreValue::Boolean(Boolean(true));
        let result = core_value.try_as::<bool>();
        assert_eq!(*result.unwrap(), true);
        
        let result_mut = core_value.try_as_mut::<bool>();
        assert_eq!(*result_mut.unwrap(), true);
        
        let result_into: Result<bool, ()> = core_value.try_into();
        assert_eq!(result_into.unwrap(), true);
    }
}