use crate::{
    ast::expressions::DatexExpressionData, prelude::*,
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl<T> ToDatexExpressionData for Box<T>
where
    T: ?Sized + ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        (**self).to_datex_expression_data()
    }
}
