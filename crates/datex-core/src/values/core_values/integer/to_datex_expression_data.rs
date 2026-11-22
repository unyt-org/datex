use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::integer::Integer,
};

impl ToDatexExpressionData for Integer {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Integer(self.clone())
    }
}
