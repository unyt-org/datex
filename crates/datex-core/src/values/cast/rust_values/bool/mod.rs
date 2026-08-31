pub mod try_from_core_value;

use crate::{
    datex_proxy::TryFromDatexValueError,
    prelude::*,
    traits::value_access::ValueAccess,
};

#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use crate::{
        ast::expressions::DatexExpressionData,
        traits::to_datex_expression_data::ToDatexExpressionData,
        values::core_values::boolean::Boolean,
    };

    impl ToDatexExpressionData for bool {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::Boolean(Boolean(*self))
        }
    }
}

impl ValueAccess for bool {}