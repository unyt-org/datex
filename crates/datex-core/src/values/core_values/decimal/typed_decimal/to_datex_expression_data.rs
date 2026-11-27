use crate::{
    ast::expressions::NormalExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

impl ToDatexExpressionData for TypedDecimal {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        NormalExpressionData::TypedDecimal(self.clone())
    }
}
