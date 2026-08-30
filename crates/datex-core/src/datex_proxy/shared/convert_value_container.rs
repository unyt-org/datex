use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::{
        value_container::ValueContainer,
    },
};
use crate::datex_proxy::shared::Shared;
use crate::preludes::derive::{BorrowedValueContainer, DatexNative};
use crate::traits::convert_value_container::ConvertValueContainer;

impl<T> ConvertValueContainer for Shared<T> 
where T: ConvertValueContainer + DatexNative
{
    fn to_value_container(
        self,
        _cache: &mut SharedReferencesCache,
    ) -> ValueContainer {
        ValueContainer::Shared(self.container)
    }

    fn as_borrowed_value_container<'a>(&'a self, cache: &mut SharedReferencesCache) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Shared(self.container.clone())
    }

    fn try_from_value_container(value_container: ValueContainer) -> Result<Self, ()>
    where
        Self: Sized
    {
        match value_container {
            ValueContainer::Shared(container) => {
                Shared::try_from(container).map_err(|_| ())
            }
            _ => Err(()),
        }
    }
}
