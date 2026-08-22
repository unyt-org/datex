use crate::ast::expressions::DatexExpressionData;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_values::native::NativeCoreValue;

impl ToDatexExpressionData for NativeCoreValue {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        self.value.to_datex_expression_data()
    }
}