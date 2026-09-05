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
            borrowed_value::{BorrowedCoreValue, BorrowedValue},
        },
        value_container::value_key::BorrowedValueKey,
    },
};
use core::cell::Ref;
use crate::values::value::value_classification::ValueClassification;

impl ValueAccess for Value {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        match &self.inner {
            CoreValue::Map(map) => map.try_get_property(key, cache),
            CoreValue::List(list) => list.try_get_property(key, cache),
            CoreValue::Type(Type::Entity(container)) => {
                if let Some(key) = key.try_as_text() {
                    let reference = Ref::filter_map(
                        container.entity_definition(),
                        |entity_definition| {
                            entity_definition.try_get_property(key)
                        },
                    )
                    .map_err(|_| {
                        AccessError::KeyNotFound(KeyNotFoundError::new(
                            key.into(),
                        ))
                    })?;
                    Ok(BorrowedValueContainer::Local(BorrowedValue {
                        inner: BorrowedCoreValue::Callable(Goat::Ref(
                            reference,
                        )),
                        classification: ValueClassification::None,
                    }))
                } else {
                    Err(AccessError::InvalidIndexKey)
                }
            }
            CoreValue::Native(native) => native.try_get_property(key, cache),
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
        cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        match &mut self.inner {
            CoreValue::Map(map) => map.try_get_property_mut(key, cache),
            CoreValue::List(list) => list.try_get_property_mut(key, cache),
            CoreValue::Native(native) => {
                native.try_get_property_mut(key, cache)
            }
            _ => {
                // If the value is not a map, we cannot get a property
                Err(AccessError::InvalidOperation(
                    "Cannot get property".to_string(),
                ))
            }
        }
    }
}
