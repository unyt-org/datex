use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

impl ToDatexExpressionData for TypedDecimal {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::TypedDecimal(self.clone())
    }
}
