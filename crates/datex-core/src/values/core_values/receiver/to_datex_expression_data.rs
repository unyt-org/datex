use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::receiver::Receiver,
};

impl<T> ToDatexExpressionData for Receiver<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Receiver
    }
}
