use crate::{
    ast::expressions::NormalExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::integer::typed_integer::TypedInteger,
};

impl ToDatexExpressionData for TypedInteger {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        NormalExpressionData::TypedInteger(self.clone())
    }
}
