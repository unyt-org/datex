use crate::{
    ast::expressions::DatexExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::integer::Integer,
};

impl ToDatexExpressionData for Integer {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Integer(self.clone())
    }
}
