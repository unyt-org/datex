use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::{AccessError, KeyNotFoundError},
    traits::value_access::ValueAccess,
    values::{
        core_value::CoreValue,
        value::{
            Value,
        },
        value_container::value_key::BorrowedValueKey,
    },
};
use core::cell::{Ref, RefCell};
use crate::values::value_container::borrowed_value_container::{BorrowedValueContainer, BorrowedValueContainerMut};

impl ValueAccess for Value {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        match &self {
            Value::Core(core) => core.try_get_property(key),
            Value::Native(native) => native.try_get_property(key),
            _ => {
                // If the value is not a map, we cannot get a property
                Err(AccessError::InvalidOperation(
                    "Cannot get property".to_string(),
                ))
            }
        }
    }

    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        match self {
            Value::Core(core) => core.try_get_property_mut(key),
            Value::Native(native) => native.try_get_property_mut(key),
            _ => {
                // If the value is not a map, we cannot get a property
                Err(AccessError::InvalidOperation(
                    "Cannot get property".to_string(),
                ))
            }
        }
    }
}
