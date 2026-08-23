use core::cell::Ref;
use crate::shared_values::errors::{AccessError, KeyNotFoundError};
use crate::traits::value_access::ValueAccess;
use crate::types::r#type::Type;
use crate::values::core_value::CoreValue;
use crate::values::value::{Value, ValueContainerOrCallable};
use crate::values::value_container::value_key::BorrowedValueKey;
use crate::values::value_container::ValueContainer;

impl ValueAccess for Value {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
    ) -> Result<ValueContainerOrCallable<'_>, AccessError> {
        match &self.inner {
            CoreValue::Map(map) => map.try_get_property(key),
            CoreValue::List(list) => list.try_get_property(key),
            CoreValue::Type(Type::Entity(container)) => {
                if let Some(key) = key.try_as_text() {
                    Ok(ValueContainerOrCallable::Callable(
                        Ref::filter_map(
                            container.entity_definition(),
                            |entity_definition| {
                                entity_definition.try_get_property(key)
                            },
                        )
                            .map_err(|_| {
                                AccessError::KeyNotFound(KeyNotFoundError::new(
                                    key.into(),
                                ))
                            })?,
                    ))
                } else {
                    Err(AccessError::InvalidIndexKey)
                }
            }
            CoreValue::Native(native) => native.try_get_property(key),
            _ => {
                // If the value is not an map, we cannot get a property
                Err(AccessError::InvalidOperation(
                    "Cannot get property".to_string(),
                ))
            }
        }
    }

    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
    ) -> Result<&mut ValueContainer, AccessError> {
        match &mut self.inner {
            CoreValue::Map(map) => map.try_get_property_mut(key),
            CoreValue::List(list) => list.try_get_property_mut(key),
            CoreValue::Native(native) => native.try_get_property_mut(key),
            _ => {
                // If the value is not an map, we cannot get a property
                Err(AccessError::InvalidOperation(
                    "Cannot get property".to_string(),
                ))
            }
        }
    }
}