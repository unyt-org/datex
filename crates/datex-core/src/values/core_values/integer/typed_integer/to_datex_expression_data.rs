use crate::{
    ast::expressions::DataExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::integer::typed_integer::TypedInteger,
};

impl ToDatexExpressionData for TypedInteger {
    fn to_datex_expression_data(&self) -> DataExpressionData {
        DataExpressionData::TypedInteger(self.clone())
    }
}
