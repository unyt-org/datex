use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::AccessError,
    traits::value_access::ValueAccess,
    values::{
        borrowed_value_container::{
            AsBorrowed, AsBorrowedMut, BorrowedValueContainer,
            BorrowedValueContainerMut,
        },
        core_values::list::List,
        value_container::value_key::BorrowedValueKey,
    },
};

impl ValueAccess for List {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        if let Some(index) = key.try_as_index() {
            Ok(self.try_get(index)?.as_borrowed())
        } else {
            Err(AccessError::InvalidIndexKey)
        }
    }

    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        if let Some(index) = key.try_as_index() {
            Ok(self.try_get_mut(index)?.as_borrowed_mut())
        } else {
            Err(AccessError::InvalidIndexKey)
        }
    }
}
