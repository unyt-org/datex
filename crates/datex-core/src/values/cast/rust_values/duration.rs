#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use core::time::Duration;
    use crate::ast::expressions::DatexExpressionData;
    use crate::traits::to_datex_expression_data::ToDatexExpressionData;
    use crate::values::core_values::integer::Integer;

    impl ToDatexExpressionData for Duration {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            // TODO: use amount once implemented
            DatexExpressionData::Integer(Integer::from(self.as_millis()))
        }
    }
}
