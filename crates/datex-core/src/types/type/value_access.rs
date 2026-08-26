use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::{AccessError, KeyNotFoundError},
    traits::value_access::ValueAccess,
    types::r#type::Type,
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value_container::value_key::BorrowedValueKey,
    },
};

impl ValueAccess for Type {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        if let Type::Entity(container) = self {
            container.try_get_property(key, cache)
        } else {
            Err(AccessError::InvalidOperation(
                "Cannot get property from non-entity type".to_string(),
            ))
        }
    }
}
