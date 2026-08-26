use crate::{
    datex_proxy::TryFromDatexValueError,
    prelude::*,
    traits::value_access::ValueAccess,
    utils::{goat::Goat, goat_mut::GoatMut},
    values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut},
};
use num_traits::ToPrimitive;

#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use crate::{
        ast::expressions::DatexExpressionData,
        traits::to_datex_expression_data::ToDatexExpressionData,
        values::core_values::integer::typed_integer::TypedInteger,
    };

    impl ToDatexExpressionData for u8 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U8(*self))
        }
    }

    impl ToDatexExpressionData for u16 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U16(*self))
        }
    }

    impl ToDatexExpressionData for u32 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U32(*self))
        }
    }

    impl ToDatexExpressionData for u64 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U64(*self))
        }
    }

    impl ToDatexExpressionData for u128 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::U128(*self))
        }
    }

    impl ToDatexExpressionData for i8 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I8(*self))
        }
    }

    impl ToDatexExpressionData for i16 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I16(*self))
        }
    }

    impl ToDatexExpressionData for i32 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I32(*self))
        }
    }

    impl ToDatexExpressionData for i64 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I64(*self))
        }
    }

    impl ToDatexExpressionData for i128 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedInteger(TypedInteger::I128(*self))
        }
    }

    impl ToDatexExpressionData for usize {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            #[cfg(target_pointer_width = "32")]
            {
                DatexExpressionData::TypedInteger(TypedInteger::U32(
                    *self as u32,
                ))
            }
            #[cfg(target_pointer_width = "64")]
            {
                DatexExpressionData::TypedInteger(TypedInteger::U64(
                    *self as u64,
                ))
            }
        }
    }

    impl ToDatexExpressionData for isize {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            #[cfg(target_pointer_width = "32")]
            {
                DatexExpressionData::TypedInteger(TypedInteger::I32(
                    *self as i32,
                ))
            }
            #[cfg(target_pointer_width = "64")]
            {
                DatexExpressionData::TypedInteger(TypedInteger::I64(
                    *self as i64,
                ))
            }
        }
    }
}

impl ValueAccess for u8 {}
impl ValueAccess for u16 {}
impl ValueAccess for u32 {}
impl ValueAccess for u64 {}
impl ValueAccess for u128 {}
impl ValueAccess for i8 {}
impl ValueAccess for i16 {}
impl ValueAccess for i32 {}
impl ValueAccess for i64 {}
impl ValueAccess for i128 {}
impl ValueAccess for usize {}
impl ValueAccess for isize {}

macro_rules! impl_try_from_borrowed_integer_goat {
    ($(($ty:ty, $borrow_as:ident, $borrow_mut_as:ident)),* $(,)?) => {
        $(
            impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, $ty> {
                type Error = TryFromDatexValueError;

                fn try_from(
                    value: BorrowedCoreValue<'a>,
                ) -> Result<Self, Self::Error> {
                    match value {
                        BorrowedCoreValue::TypedInteger(value) => {
                            value
                                .filter_map(|v| v.$borrow_as())
                                .ok_or_else(|| {
                                    TryFromDatexValueError(
                                        format!(
                                            "Cannot cast value to {}",
                                            stringify!($ty)
                                        )
                                    )
                                })
                        }
                        _ => Err(TryFromDatexValueError(format!(
                            "Cannot cast BorrowedCoreValue to {}",
                            stringify!($ty)
                        ))),
                    }
                }
            }

            impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, $ty> {
                type Error = TryFromDatexValueError;

                fn try_from(
                    value: BorrowedCoreValueMut<'a>,
                ) -> Result<Self, Self::Error> {
                    match value {
                        BorrowedCoreValueMut::TypedInteger(value) => {
                            value
                                .filter_map(|v| v.$borrow_mut_as())
                                .ok_or_else(|| {
                                    TryFromDatexValueError(
                                        format!(
                                            "Cannot cast value to {}",
                                            stringify!($ty)
                                        )
                                    )
                                })
                        }
                        _ => Err(TryFromDatexValueError(format!(
                            "Cannot cast BorrowedCoreValueMut to {}",
                            stringify!($ty)
                        ))),
                    }
                }
            }
        )*
    };
}

impl_try_from_borrowed_integer_goat!(
    (i8, borrow_as_i8, borrow_mut_as_i8),
    (i16, borrow_as_i16, borrow_mut_as_i16),
    (i32, borrow_as_i32, borrow_mut_as_i32),
    (i64, borrow_as_i64, borrow_mut_as_i64),
    (i128, borrow_as_i128, borrow_mut_as_i128),
    (u8, borrow_as_u8, borrow_mut_as_u8),
    (u16, borrow_as_u16, borrow_mut_as_u16),
    (u32, borrow_as_u32, borrow_mut_as_u32),
    (u64, borrow_as_u64, borrow_mut_as_u64),
    (u128, borrow_as_u128, borrow_mut_as_u128),
);
