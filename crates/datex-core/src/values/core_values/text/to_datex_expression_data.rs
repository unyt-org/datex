use crate::{
    ast::expressions::NormalExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::text::Text,
};

impl ToDatexExpressionData for Text {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        NormalExpressionData::Text(self.clone())
    }
}
