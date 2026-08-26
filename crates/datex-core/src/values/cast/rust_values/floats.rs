use crate::{
    datex_proxy::TryFromDatexValueError,
    prelude::*,
    traits::value_access::ValueAccess,
    utils::{goat::Goat, goat_mut::GoatMut},
    values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut},
};

#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use crate::{
        ast::expressions::DatexExpressionData,
        traits::to_datex_expression_data::ToDatexExpressionData,
        values::core_values::decimal::typed_decimal::TypedDecimal,
    };
    use ordered_float::OrderedFloat;

    impl ToDatexExpressionData for f32 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedDecimal(TypedDecimal::F32(OrderedFloat(
                *self,
            )))
        }
    }

    impl ToDatexExpressionData for f64 {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::TypedDecimal(TypedDecimal::F64(OrderedFloat(
                *self,
            )))
        }
    }
}

impl ValueAccess for f32 {}
impl ValueAccess for f64 {}

impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, f32> {
    type Error = TryFromDatexValueError;
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::TypedDecimal(value) => {
                value.filter_map(|v| v.borrow_as_f32()).ok_or_else(|| {
                    TryFromDatexValueError(
                        "Cannot cast value to f32".to_string(),
                    )
                })
            }
            _ => Err(TryFromDatexValueError(format!(
                "Cannot cast BorrowedCoreValue to {}",
                stringify!(f32)
            ))),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, f32> {
    type Error = TryFromDatexValueError;
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::TypedDecimal(value) => {
                value.filter_map(|v| v.borrow_mut_as_f32()).ok_or_else(|| {
                    TryFromDatexValueError(
                        "Cannot cast value to f32".to_string(),
                    )
                })
            }
            _ => Err(TryFromDatexValueError(format!(
                "Cannot cast BorrowedCoreValueMut to {}",
                stringify!(f32)
            ))),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, f64> {
    type Error = TryFromDatexValueError;
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::TypedDecimal(value) => {
                value.filter_map(|v| v.borrow_as_f64()).ok_or_else(|| {
                    TryFromDatexValueError(
                        "Cannot cast value to f64".to_string(),
                    )
                })
            }
            _ => Err(TryFromDatexValueError(format!(
                "Cannot cast BorrowedCoreValue to {}",
                stringify!(f64)
            ))),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, f64> {
    type Error = TryFromDatexValueError;
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::TypedDecimal(value) => {
                value.filter_map(|v| v.borrow_mut_as_f64()).ok_or_else(|| {
                    TryFromDatexValueError(
                        "Cannot cast value to f64".to_string(),
                    )
                })
            }
            _ => Err(TryFromDatexValueError(format!(
                "Cannot cast BorrowedCoreValueMut to {}",
                stringify!(f64)
            ))),
        }
    }
}
