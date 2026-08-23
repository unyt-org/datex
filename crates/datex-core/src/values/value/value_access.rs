use core::cell::Ref;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::shared_values::errors::{AccessError, KeyNotFoundError};
use crate::traits::value_access::ValueAccess;
use crate::types::r#type::Type;
use crate::utils::goat::Goat;
use crate::values::borrowed_value_container::{BorrowedValueContainer, BorrowedValueContainerMut};
use crate::values::core_value::CoreValue;
use crate::values::value::{Value};
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedValue};
use crate::values::value_container::value_key::BorrowedValueKey;
use crate::prelude::*;

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
                    Ok(BorrowedValueContainer::Local(
                        BorrowedValue {
                            inner: BorrowedCoreValue::Callable(Goat::Ref(reference)),
                            custom_type: None,
                        }
                    ))
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
            CoreValue::Native(native) => native.try_get_property_mut(key, cache),
            _ => {
                // If the value is not a map, we cannot get a property
                Err(AccessError::InvalidOperation(
                    "Cannot get property".to_string(),
                ))
            }
        }
    }
}