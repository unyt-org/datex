use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::{
        value_container::ValueContainer,
    },
};
use crate::preludes::derive::{BorrowedValueContainer, DatexNative};
use crate::traits::convert_value_container::ConvertValueContainer;

impl ConvertValueContainer for ValueContainer {
    fn to_value_container(
        self,
        _cache: &mut SharedReferencesCache,
    ) -> ValueContainer {
        self
    }

    fn as_borrowed_value_container(&self, _cache: &mut SharedReferencesCache) -> BorrowedValueContainer {
        BorrowedValueContainer::from(self)
    }

    fn try_from_value_container(value_container: ValueContainer) -> Result<Self, ()>
    where
        Self: Sized
    {
        Ok(value_container)
    }
}
