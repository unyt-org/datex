use crate::{
    ast::expressions::DatexExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::integer::typed_integer::TypedInteger,
};

impl ToDatexExpressionData for TypedInteger {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        DatexExpressionData::TypedInteger(self.clone())
    }
}
