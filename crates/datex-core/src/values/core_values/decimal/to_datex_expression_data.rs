use crate::{
    ast::expressions::DatexExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::decimal::Decimal,
};

impl ToDatexExpressionData for Decimal {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::Decimal(self.clone())
    }
}
