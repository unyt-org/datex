
use crate::{
    preludes::derive::BorrowedValueContainer,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::convert_value_container::ConvertValueContainer,
    values::value_container::ValueContainer,
};
use crate::shared_values::{SharedContainer};

impl ConvertValueContainer for SharedContainer {
    fn to_value_container(
        self,
        _cache: &mut SharedReferencesCache,
    ) -> ValueContainer {
        ValueContainer::Shared(self)
    }

    fn as_borrowed_value_container(
        &self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer<'_> {
        BorrowedValueContainer::Shared(self.clone())
    }

    fn try_from_value_container(
        value_container: ValueContainer,
    ) -> Result<Self, ValueContainer>
    where
        Self: Sized,
    {
        match value_container {
            ValueContainer::Shared(shared_container) => {
                Ok(shared_container)
            }
            _ => Err(value_container),
        }
    }

    fn try_borrow_from_value_container(
        value_container: &ValueContainer,
    ) -> Result<&Self, ()>
    where
        Self: Sized,
    {
        match value_container {
            ValueContainer::Shared(shared_container) => {
                Ok(shared_container)
            }
            _ => Err(()),
        }
    }

    fn try_borrow_mut_from_value_container(
        value_container: &mut ValueContainer,
    ) -> Result<&mut Self, ()>
    where
        Self: Sized,
    {
        match value_container {
            ValueContainer::Shared(shared_container) => {
                Ok(shared_container)
            }
            _ => Err(()),
        }
    }
}
