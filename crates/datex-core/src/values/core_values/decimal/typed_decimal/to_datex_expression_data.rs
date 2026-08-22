use crate::ast::expressions::{DatexExpressionData};
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

impl ToDatexExpressionData for TypedDecimal {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::TypedDecimal(self.clone())
    }
}