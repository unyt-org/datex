use crate::ast::expressions::{DatexExpressionData};
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::values::core_value::CoreValue;

impl ToDatexExpressionData for CoreValue {
    fn to_datex_expression_data(&self) -> DatexExpressionData {
        match &self {
            CoreValue::Integer(integer) => integer.to_datex_expression_data(),
            CoreValue::TypedInteger(typed_integer) => typed_integer.to_datex_expression_data(),
            CoreValue::Decimal(decimal) => decimal.to_datex_expression_data(),
            CoreValue::TypedDecimal(typed_decimal) => typed_decimal.to_datex_expression_data(),
            CoreValue::Boolean(boolean) => boolean.to_datex_expression_data(),
            CoreValue::Text(text) => text.to_datex_expression_data(),
            CoreValue::Range(range) => range.to_datex_expression_data(),
            CoreValue::Endpoint(endpoint) => endpoint.to_datex_expression_data(),
            CoreValue::Null => DatexExpressionData::Null,
            CoreValue::List(list) => list.to_datex_expression_data(),
            CoreValue::Map(map) => map.to_datex_expression_data(),
            CoreValue::Type(type_value) => type_value.to_datex_expression_data(),
            CoreValue::Callable(callable) => callable.to_datex_expression_data(),
            CoreValue::EntityTypeDefinition(entity_type_definition) => entity_type_definition.to_datex_expression_data(),
            CoreValue::Uninitialized => todo!(),
            CoreValue::Box(inner) => inner.to_datex_expression_data(),
            CoreValue::Native(value) => value.to_datex_expression_data(),
        }
    }
}