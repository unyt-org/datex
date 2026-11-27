use crate::{
    ast::expressions::NormalExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::decimal::Decimal,
};

impl ToDatexExpressionData for Decimal {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        NormalExpressionData::Decimal(self.clone())
    }
}
