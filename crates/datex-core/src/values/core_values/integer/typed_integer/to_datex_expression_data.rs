use crate::ast::expressions::{DatexExpressionData};
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::integer::typed_integer::TypedInteger;

impl ToDatexExpressionData for TypedInteger {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::TypedInteger(self.clone())
    }
}