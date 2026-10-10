use core::cell::RefCell;

use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::AccessError,
    traits::value_access::ValueAccess,
    values::{
        core_values::list::List,
        value_container::value_key::BorrowedValueKey,
    },
};
use crate::values::value_container::borrowed_value_container::{BorrowedValueContainer, BorrowedValueContainerMut};

impl ValueAccess for List {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        if let Some(index) = key.try_as_index() {
            Ok(self.try_get(index)?.into())
        } else {
            Err(AccessError::InvalidIndexKey)
        }
    }

    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        if let Some(index) = key.try_as_index() {
            Ok(self.try_get_mut(index)?.into())
        } else {
            Err(AccessError::InvalidIndexKey)
        }
    }
}
