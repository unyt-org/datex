use crate::ast::expressions::{DatexExpressionData};
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::integer::Integer;

impl ToDatexExpressionData for Integer {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Integer(self.clone())
    }
}