use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::{AccessError, KeyNotFoundError},
    traits::value_access::ValueAccess,
    types::entity_type::EntityType,
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value::{
            borrowed_value::{
                BorrowedValue, borrowed_core_value::BorrowedCoreValue,
                borrowed_core_value_with_classification::BorrowedCoreValueWithClassification,
            },
            value_classification::ValueClassification,
        },
        value_container::value_key::BorrowedValueKey,
    },
};
use core::cell::{Ref, RefCell};

impl ValueAccess for EntityType {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        _cache: &RefCell<SharedReferencesCache>,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        if let Some(key) = key.try_as_text() {
            let callable_ref = Ref::filter_map(
                self.entity_definition(),
                |entity_definition| entity_definition.try_get_property(key),
            )
            .map_err(|_| {
                AccessError::KeyNotFound(KeyNotFoundError::new(
                    key.to_string().into(),
                ))
            })?;
            Ok(BorrowedValueContainer::Local(BorrowedValue::Core(
                BorrowedCoreValueWithClassification {
                    inner: BorrowedCoreValue::Callable(callable_ref.into()),
                    classification: ValueClassification::new_unclassified(),
                },
            )))
        } else {
            Err(AccessError::InvalidIndexKey)
        }
    }
}
