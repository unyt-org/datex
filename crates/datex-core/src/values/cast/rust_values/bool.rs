use crate::datex_proxy::TryFromDatexValueError;
use crate::traits::value_access::ValueAccess;
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut};
use crate::prelude::*;

#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use crate::ast::expressions::DatexExpressionData;
    use crate::traits::to_datex_expression_data::ToDatexExpressionData;
    use crate::values::core_values::boolean::Boolean;

    impl ToDatexExpressionData for bool {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::Boolean(Boolean(*self))
        }
    }
}

impl ValueAccess for bool {}

impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, bool> {
    type Error = TryFromDatexValueError;
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::Boolean(v) => Ok(v.map(|v| &v.0)),
            _ => Err(TryFromDatexValueError(format!("Cannot cast BorrowedCoreValue to {}", stringify!(bool)))),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, bool> {
    type Error = TryFromDatexValueError;
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::Boolean(v) => Ok(v.map(|v| &mut v.0)),
            _ => Err(TryFromDatexValueError(format!("Cannot cast BorrowedCoreValueMut to {}", stringify!(bool)))),
        }
    }
}