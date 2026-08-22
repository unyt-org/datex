#[cfg(feature = "decompiler")]
mod to_datex_expression_data {
    use crate::ast::expressions::DatexExpressionData;
    use crate::traits::to_datex_expression_data::ToDatexExpressionData;
    use crate::values::core_values::text::Text;

    impl ToDatexExpressionData for String {
        fn to_datex_expression_data(&self) -> DatexExpressionData {
            DatexExpressionData::Text(Text(self.clone()))
        }
    }
}