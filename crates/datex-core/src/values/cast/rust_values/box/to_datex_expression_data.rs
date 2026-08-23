use core::ops::Deref;
use crate::prelude::*;
use crate::ast::expressions::{DatexExpressionData, Statements};
use crate::ast::spanned::Spanned;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;

impl<T> ToDatexExpressionData for Box<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(
        &self,
    ) -> DatexExpressionData {
        self.deref().to_datex_expression_data()
    }
}