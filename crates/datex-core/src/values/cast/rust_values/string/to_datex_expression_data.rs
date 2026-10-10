use crate::{
    ast::expressions::DatexExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::text::Text,
};

impl<'a> ToDatexExpressionData for &'a str {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Text(Text(self.to_string()))
    }
}

impl ToDatexExpressionData for String {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Text(Text(self.clone()))
    }
}
