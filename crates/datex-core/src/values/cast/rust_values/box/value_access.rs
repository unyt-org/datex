use crate::{
    prelude::*,
    preludes::derive::{
        AccessError, BorrowedValueContainer, BorrowedValueContainerMut,
        BorrowedValueKey, SharedReferencesCache,
    },
    traits::value_access::ValueAccess,
};
use core::ops::{Deref, DerefMut};

impl<T: ValueAccess> ValueAccess for Box<T> {
    fn try_get_property(
        &self,
        _key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        self.deref().try_get_property(_key, _cache)
    }
    fn try_get_property_mut(
        &mut self,
        _key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        self.deref_mut().try_get_property_mut(_key, _cache)
    }
}
