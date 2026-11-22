use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::instant::Instant,
};

impl ToDatexExpressionData for Instant {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::Instant(*self)
    }
}
