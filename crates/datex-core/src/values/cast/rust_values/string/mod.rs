mod try_from_core_value;

use crate::{
    prelude::*,
    traits::value_access::ValueAccess,
};
use crate::traits::datex_hash::impl_datex_hash;

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

impl_datex_hash!(String);