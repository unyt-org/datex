use crate::{
    collections::HashMap,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::AccessError,
    traits::value_access::ValueAccess,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_values::native::DatexNative,
        value_container::value_key::BorrowedValueKey,
    },
};
use core::hash::Hash;

impl<K, V> ValueAccess for HashMap<K, V>
where
    K: DatexNative + Eq + Hash,
    V: DatexNative,
{
    fn try_get_property(
        &self,
        _key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        todo!()
    }

    fn try_get_property_mut(
        &mut self,
        _key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        todo!()
    }
}
