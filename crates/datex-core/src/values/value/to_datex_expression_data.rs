use crate::{
    ast::expressions::DatexExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::value::{Value, borrowed_value::BorrowedValue},
};

impl ToDatexExpressionData for Value {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        BorrowedValue::from(self).to_datex_expression_data()
    }
}
