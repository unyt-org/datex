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
        ast::expressions::DatexExpressionData, prelude::*,
        traits::to_datex_expression_data::ToDatexExpressionData,
        values::core_values::text::Text,
    };

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
            BorrowedCoreValue::Text(value) => Ok(value.map(|v| &v.0)),
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
            BorrowedCoreValueMut::Text(value) => Ok(value.map(|v| &mut v.0)),
            _ => Err(TryFromDatexValueError(format!(
                "Cannot cast BorrowedCoreValue to {}",
                stringify!(String)
            ))),
        }
    }
}
