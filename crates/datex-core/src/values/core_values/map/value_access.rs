use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::shared_values::errors::{AccessError};
use crate::traits::value_access::ValueAccess;
use crate::values::borrowed_value_container::{BorrowedValueContainer, BorrowedValueContainerMut};
use crate::values::core_values::map::Map;
use crate::values::value_container::value_key::BorrowedValueKey;

impl ValueAccess for Map {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        Ok(self.try_get(key)?.into())
    }

    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
        cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        Ok(self.try_get_mut(key)?.into())
    }
}