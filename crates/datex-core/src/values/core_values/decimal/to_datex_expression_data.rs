use crate::ast::expressions::{DatexExpressionData};
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::decimal::Decimal;

impl ToDatexExpressionData for Decimal {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Decimal(self.clone())
    }
}