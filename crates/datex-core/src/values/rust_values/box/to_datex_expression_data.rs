use crate::{
    ast::expressions::NormalExpressionData, prelude::*,
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl<T> ToDatexExpressionData for Box<T>
where
    T: ?Sized + ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        (**self).to_datex_expression_data()
    }
}
