use crate::ast::expressions::{DatexExpressionData};
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::endpoint::Endpoint;

impl ToDatexExpressionData for Endpoint {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Endpoint(self.clone())
    }
}