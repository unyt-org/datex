use crate::{
    ast::expressions::NormalExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::endpoint::Endpoint,
};

impl ToDatexExpressionData for Endpoint {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        NormalExpressionData::Endpoint(self.clone())
    }
}
