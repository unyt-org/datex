use core::cell::Ref;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::shared_values::errors::{AccessError, KeyNotFoundError};
use crate::traits::value_access::ValueAccess;
use crate::types::shared_container_containing_entity_type::SharedContainerContainingEntityType;
use crate::utils::goat::Goat;
use crate::values::borrowed_value_container::BorrowedValueContainer;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedValue};
use crate::values::value_container::value_key::BorrowedValueKey;

impl ValueAccess for SharedContainerContainingEntityType {
    fn try_get_property(&self, key: BorrowedValueKey, _cache: &mut SharedReferencesCache) -> Result<BorrowedValueContainer<'_>, AccessError> {
        if let Some(key) = key.try_as_text() {
            let callable_ref = Ref::filter_map(
                self.entity_definition(),
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
                inner: BorrowedCoreValue::Native(Goat::Ref(callable_ref)),
                custom_type: None
            }))
        } else {
            Err(AccessError::InvalidIndexKey)
        }
    }
}