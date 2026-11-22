use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::boolean::Boolean,
};

impl ToDatexExpressionData for Boolean {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Boolean(self.clone())
    }
}
