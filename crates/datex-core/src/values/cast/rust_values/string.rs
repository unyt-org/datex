use crate::datex_proxy::TryFromDatexValueError;
use crate::traits::value_access::ValueAccess;
use crate::prelude::*;
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut};

#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use crate::ast::expressions::DatexExpressionData;
    use crate::traits::to_datex_expression_data::ToDatexExpressionData;
    use crate::values::core_values::text::Text;
    use crate::prelude::*;

    impl ToDatexExpressionData for String {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::Text(Text(self.clone()))
        }
    }
}

impl ValueAccess for String {}

impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, String> {
    type Error = TryFromDatexValueError;
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::Text(value) => {
                Ok(value.map(|v| &v.0))
            },
            _ => Err(TryFromDatexValueError(format!(
                "Cannot cast BorrowedCoreValue to {}",
                stringify!(String)
            ))),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, String> {
    type Error = TryFromDatexValueError;
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::Text(value) => {
                Ok(value.map(|v| &mut v.0))
            },
            _ => Err(TryFromDatexValueError(format!(
                "Cannot cast BorrowedCoreValue to {}",
                stringify!(String)
            ))),
        }
    }
}