use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl<T> ToDatexExpressionData for Option<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DataExpressionData {
        match self {
            Some(value) => value.to_datex_expression_data(),
            None => DataExpressionData::Null,
        }
    }
}
