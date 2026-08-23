use crate::traits::value_access::ValueAccess;

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