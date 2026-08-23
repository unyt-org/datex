use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::shared_values::errors::{AccessError};
use crate::traits::value_access::ValueAccess;
use crate::types::entities::entity_type_definition::EntityTypeDefinition;
use crate::values::borrowed_value_container::BorrowedValueContainer;
use crate::values::value_container::value_key::BorrowedValueKey;

impl ValueAccess for EntityTypeDefinition {
    fn try_get_property(&self, key: BorrowedValueKey, cache: &mut SharedReferencesCache) -> Result<BorrowedValueContainer<'_>, AccessError> {
        todo!()
    }
}