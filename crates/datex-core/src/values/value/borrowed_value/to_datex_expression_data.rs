use crate::{
    ast::expressions::NormalExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::value::borrowed_value::{BorrowedValue, BorrowedValueMut},
};

impl ToDatexExpressionData for BorrowedValue<'_> {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        match self {
            BorrowedValue::Core(core_value) => {
                core_value.to_datex_expression_data()
            }
            BorrowedValue::Native(native_value) => {
                native_value.to_datex_expression_data()
            }
        }
    }
}

impl ToDatexExpressionData for BorrowedValueMut<'_> {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        match self {
            BorrowedValueMut::Core(core_value) => {
                core_value.to_datex_expression_data()
            }
            BorrowedValueMut::Native(native_value) => {
                native_value.to_datex_expression_data()
            }
        }
    }
}
