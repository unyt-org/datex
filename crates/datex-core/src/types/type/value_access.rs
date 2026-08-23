use core::cell::Ref;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::shared_values::errors::{AccessError, KeyNotFoundError};
use crate::traits::value_access::ValueAccess;
use crate::types::r#type::Type;
use crate::values::borrowed_value_container::BorrowedValueContainer;
use crate::values::value_container::value_key::BorrowedValueKey;

impl ValueAccess for Type {
    fn try_get_property(&self, key: BorrowedValueKey, cache: &mut SharedReferencesCache) -> Result<BorrowedValueContainer<'_>, AccessError> {
        if let Type::Entity(container) = self {
            container.try_get_property(key, cache)
        } else {
            Err(AccessError::InvalidOperation(
                "Cannot get property from non-entity type".to_string(),
            ))
        }
    }
}