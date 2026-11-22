use crate::{
    ast::{expressions::DataExpressionData, spanned::Spanned},
    prelude::*,
    traits::to_datex_expression_data::ToDatexExpressionData,
};

impl<T> ToDatexExpressionData for Vec<T>
where
    T: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::List(
            self.iter()
                .map(|v| v.to_datex_expression_data().with_default_span())
                .collect(),
        )
    }
}
