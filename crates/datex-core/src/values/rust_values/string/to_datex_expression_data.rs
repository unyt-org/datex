use crate::{
    ast::expressions::DataExpressionData, prelude::*,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::text::Text,
};

impl ToDatexExpressionData for &str {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Text(Text::String(self.to_string()))
    }
}

impl ToDatexExpressionData for String {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Text(Text::String(self.clone()))
    }
}
