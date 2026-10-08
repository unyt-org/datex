use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::{AccessError, KeyNotFoundError},
    traits::value_access::ValueAccess,
    types::r#type::Type,
    utils::goat::Goat,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_value::CoreValue,
        value::{
            Value,
        },
        value_container::value_key::BorrowedValueKey,
    },
};
use core::cell::{Ref, RefCell};

impl ValueAccess for Value {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        match &self {
            Value::Core(core) => core.try_get_property(key, cache),
            Value::Native(native) => native.try_get_property(key, cache),
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
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        match self {
            Value::Core(core) => core.try_get_property_mut(key, cache),
            Value::Native(native) => native.try_get_property_mut(key, cache),
            _ => {
                // If the value is not a map, we cannot get a property
                Err(AccessError::InvalidOperation(
                    "Cannot get property".to_string(),
                ))
            }
        }
    }
}
