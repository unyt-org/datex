mod try_from_core_value;

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