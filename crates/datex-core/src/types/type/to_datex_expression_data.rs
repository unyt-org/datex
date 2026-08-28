use crate::{
    ast::{expressions::DatexExpressionData, spanned::Spanned},
    traits::{
        to_datex_expression_data::ToDatexExpressionData,
        to_type_expression_data::ToTypeExpressionData,
    },
    types::r#type::Type,
};

impl ToDatexExpressionData for Type {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::TypeExpression(
            self.to_type_expression_data().with_default_span(),
        )
    }
}
