use crate::{
    ast::expressions::DataExpressionData, prelude::*,
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl<T> ToDatexExpressionData for Box<T>
where
    T: ?Sized + ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DataExpressionData {
        (**self).to_datex_expression_data()
    }
}
