use crate::{
    ast::{expressions::NormalExpressionData, spanned::Spanned},
    prelude::*,
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl<T> ToDatexExpressionData for Vec<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        NormalExpressionData::List(
            self.iter()
                .map(|v| v.to_datex_expression_data().with_default_span())
                .collect(),
        )
    }
}
