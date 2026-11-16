use crate::{
    ast::{expressions::DatexExpressionData, spanned::Spanned},
    prelude::*,
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl<T> ToDatexExpressionData for Vec<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::List(
            self.iter()
                .map(|v| v.to_datex_expression_data().with_default_span())
                .collect(),
        )
    }
}
