use crate::{
    ast::expressions::DatexExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl<T> ToDatexExpressionData for Option<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        match self {
            Some(value) => value.to_datex_expression_data(),
            None => DatexExpressionData::Null,
        }
    }
}
