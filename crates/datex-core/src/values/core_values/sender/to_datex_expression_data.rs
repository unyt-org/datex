use crate::{
    ast::expressions::NormalExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::sender::Sender,
};

impl<T> ToDatexExpressionData for Sender<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        NormalExpressionData::Sender
    }
}
