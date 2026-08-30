use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::AccessError,
    traits::value_access::ValueAccess,
    values::{
        borrowed_value_container::{
            AsBorrowed, AsBorrowedMut, BorrowedValueContainer,
            BorrowedValueContainerMut,
        },
        core_values::map::Map,
        value_container::value_key::BorrowedValueKey,
    },
};

impl ValueAccess for Map {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        Ok(self.try_get(key)?.into())
    }

    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        Ok(self.try_get_mut(key)?.into())
    }
}
