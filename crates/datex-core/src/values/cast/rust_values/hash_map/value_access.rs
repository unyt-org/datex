use crate::collections::HashMap;
use core::hash::Hash;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::shared_values::errors::AccessError;
use crate::traits::value_access::ValueAccess;
use crate::values::borrowed_value_container::{BorrowedValueContainer, BorrowedValueContainerMut};
use crate::values::core_values::native::DatexNative;
use crate::values::value_container::value_key::BorrowedValueKey;

impl<K, V> ValueAccess for HashMap<K, V>
where
    K: DatexNative + Eq + Hash,
    V: DatexNative,
{
    fn try_get_property(&self, key: BorrowedValueKey, _cache: &mut SharedReferencesCache) -> Result<BorrowedValueContainer<'_>, AccessError> {
        todo!()
    }

    fn try_get_property_mut(&mut self, key: BorrowedValueKey, _cache: &mut SharedReferencesCache) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        todo!()
    }
}