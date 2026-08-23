use core::cell::Ref;
use crate::shared_values::errors::{AccessError, KeyNotFoundError};
use crate::traits::value_access::ValueAccess;
use crate::types::shared_container_containing_entity_type::SharedContainerContainingEntityType;
use crate::values::value::ValueContainerOrBorrowedValue;
use crate::values::value_container::value_key::BorrowedValueKey;

impl ValueAccess for SharedContainerContainingEntityType {
    fn try_get_property(&self, key: BorrowedValueKey) -> Result<ValueContainerOrBorrowedValue<'_>, AccessError> {
        if let Some(key) = key.try_as_text() {
            Ok(ValueContainerOrBorrowedValue::BorrowedValue(
                Ref::filter_map(
                    self.entity_definition(),
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
}