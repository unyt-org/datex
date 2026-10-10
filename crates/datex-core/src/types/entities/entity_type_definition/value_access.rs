use core::cell::RefCell;

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::AccessError,
    traits::value_access::ValueAccess,
    types::entities::entity_type_definition::EntityTypeDefinition,
    values::{
        value_container::value_key::BorrowedValueKey,
    },
};
use crate::values::value_container::borrowed_value_container::BorrowedValueContainer;

impl ValueAccess for EntityTypeDefinition {
    fn try_get_property(
        &self,
        _key: BorrowedValueKey,
        _cache: &RefCell<SharedReferencesCache>,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        todo!()
    }
}
