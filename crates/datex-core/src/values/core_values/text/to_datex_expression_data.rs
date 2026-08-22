use crate::ast::expressions::{DatexExpressionData};
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::text::Text;

impl ToDatexExpressionData for Text {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Text(self.clone())
    }
}