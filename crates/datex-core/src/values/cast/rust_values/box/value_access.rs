use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::AccessError,
    traits::value_access::ValueAccess,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        value_container::value_key::BorrowedValueKey,
    },
};
use core::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

impl<T: ValueAccess> ValueAccess for Box<T> {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        self.deref().try_get_property(key, cache)
    }
    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        self.deref_mut().try_get_property_mut(key, cache)
    }
}
