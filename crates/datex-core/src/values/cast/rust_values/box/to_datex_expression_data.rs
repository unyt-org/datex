use crate::{
    ast::expressions::DatexExpressionData, prelude::*,
    traits::to_datex_expression_data::ToDatexExpressionData,
};
use core::ops::Deref;

impl<T> ToDatexExpressionData for Box<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        self.deref().to_datex_expression_data()
    }
}
