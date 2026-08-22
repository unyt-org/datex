use crate::ast::expressions::{DatexExpressionData};
use crate::ast::spanned::Spanned;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::traits::to_type_expression_data::ToTypeExpressionData;
use crate::types::r#type::Type;

impl ToDatexExpressionData for Type {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::TypeExpression(
            self.to_type_expression_data().with_default_span(),
        )
    }
}