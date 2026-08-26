use crate::{traits::try_clone::TryClone, values::core_value::CoreValue};

impl TryClone for CoreValue {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        match self {
            CoreValue::Uninitialized => Ok(CoreValue::Uninitialized),
            CoreValue::Null => Ok(CoreValue::Null),
            CoreValue::Boolean(bool_value) => bool_value.try_clone(),
            CoreValue::Integer(int_value) => int_value.try_clone(),
            CoreValue::TypedInteger(typed_int_value) => {
                typed_int_value.try_clone()
            }
            CoreValue::Decimal(decimal_value) => decimal_value.try_clone(),
            CoreValue::TypedDecimal(typed_decimal_value) => {
                typed_decimal_value.try_clone()
            }
            CoreValue::Text(text_value) => text_value.try_clone(),
            CoreValue::Endpoint(endpoint_value) => endpoint_value.try_clone(),
            CoreValue::List(list_value) => list_value.try_clone(),
            CoreValue::Map(map_value) => map_value.try_clone(),
            CoreValue::Type(type_value) => type_value.try_clone(),
            CoreValue::EntityTypeDefinition(entity_type_def) => {
                entity_type_def.try_clone()
            }
            CoreValue::Callable(callable_value) => callable_value.try_clone(),
            CoreValue::Range(range_value) => range_value.try_clone(),
            CoreValue::Box(box_value) => box_value.try_clone(),
            CoreValue::Native(native_value) => native_value.try_clone(),
        }
    }
}
