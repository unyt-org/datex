use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::endpoint::Endpoint,
};

impl ToDatexExpressionData for Endpoint {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Endpoint(self.clone())
    }
}
