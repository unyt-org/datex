use crate::{
    ast::expressions::NormalExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl<T> ToDatexExpressionData for Option<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        match self {
            Some(value) => value.to_datex_expression_data(),
            None => NormalExpressionData::Null,
        }
    }
}
