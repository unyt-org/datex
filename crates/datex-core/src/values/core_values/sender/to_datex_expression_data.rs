use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::sender::Sender,
};

impl<T> ToDatexExpressionData for Sender<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Sender
    }
}
