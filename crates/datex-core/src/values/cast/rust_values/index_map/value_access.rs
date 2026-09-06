use crate::{
    random::RandomState,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::AccessError,
    traits::{
        convert_value_container::ConvertValueContainer,
        value_access::ValueAccess,
    },
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        value_container::value_key::BorrowedValueKey,
    },
};
use core::hash::Hash;
use indexmap::IndexMap;

impl<K, V> ValueAccess for IndexMap<K, V, RandomState>
where
    K: ConvertValueContainer + Eq + Hash,
    V: ConvertValueContainer,
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
