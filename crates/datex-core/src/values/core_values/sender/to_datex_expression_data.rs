use crate::{
    ast::{expressions::DatexExpressionData},
    traits::to_datex_expression_data::ToDatexExpressionData,
};
use crate::values::core_values::sender::Sender;

impl<T> ToDatexExpressionData for Sender<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Sender
    }
}
