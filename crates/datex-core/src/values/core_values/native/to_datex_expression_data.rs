use crate::{
    ast::expressions::DatexExpressionData,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::core_values::native::NativeCoreValue,
};

impl ToDatexExpressionData for NativeCoreValue {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        self.value.to_datex_expression_data()
    }
}
