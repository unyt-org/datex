use crate::{
    ast::expressions::NormalExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::value::{Value, borrowed_value::BorrowedValue},
};

impl ToDatexExpressionData for Value {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        BorrowedValue::from(self).to_datex_expression_data()
    }
}
