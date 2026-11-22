use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::text::Text,
};

impl ToDatexExpressionData for Text {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Text(self.clone())
    }
}
