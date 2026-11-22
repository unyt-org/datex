use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::value::{Value, borrowed_value::BorrowedValue},
};

impl ToDatexExpressionData for Value {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        BorrowedValue::from(self).to_datex_expression_data()
    }
}
