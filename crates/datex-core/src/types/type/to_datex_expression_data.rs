use crate::{
    ast::{expressions::DataExpressionData, spanned::Spanned},
    traits::{
        to_datex_expression_data::ToDatexExpressionData,
        to_type_expression_data::ToTypeExpressionData,
    },
    types::r#type::Type,
};

impl ToDatexExpressionData for Type {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::TypeExpression(
            self.to_type_expression_data().with_default_span(),
        )
    }
}
