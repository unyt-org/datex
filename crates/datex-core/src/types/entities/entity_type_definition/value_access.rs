use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::AccessError,
    traits::value_access::ValueAccess,
    types::entities::entity_type_definition::EntityTypeDefinition,
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value_container::value_key::BorrowedValueKey,
    },
};

impl ValueAccess for EntityTypeDefinition {
    fn try_get_property(
        &self,
        _key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        todo!()
    }
}
