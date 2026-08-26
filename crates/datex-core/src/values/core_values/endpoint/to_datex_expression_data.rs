use crate::{
    ast::expressions::DatexExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::endpoint::Endpoint,
};

impl ToDatexExpressionData for Endpoint {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Endpoint(self.clone())
    }
}
