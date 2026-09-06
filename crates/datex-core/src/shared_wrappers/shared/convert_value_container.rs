use crate::{
    preludes::derive::{BorrowedValueContainer, DatexNative},
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_wrappers::shared::Shared,
    traits::convert_value_container::ConvertValueContainer,
    values::value_container::ValueContainer,
};

impl<T> ConvertValueContainer for Shared<T>
where
    T: ConvertValueContainer + DatexNative,
{
    fn to_value_container(
        self,
        _cache: &mut SharedReferencesCache,
    ) -> ValueContainer {
        ValueContainer::Shared(self.container)
    }

    fn as_borrowed_value_container<'a>(
        &'a self,
        _cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer<'a> {
        BorrowedValueContainer::Shared(self.container.clone())
    }

    fn try_from_value_container(
        value_container: ValueContainer,
    ) -> Result<Self, ValueContainer>
    where
        Self: Sized,
    {
        match value_container {
            ValueContainer::Shared(container) => {
                // TODO: no clone?
                Shared::try_from(container.clone())
                    .map_err(|_| ValueContainer::Shared(container))
            }
            _ => Err(value_container),
        }
    }

    fn try_borrow_from_value_container(
        _value_container: &ValueContainer,
    ) -> Result<&Self, ()>
    where
        Self: Sized,
    {
        Err(())
    }

    fn try_borrow_mut_from_value_container(
        _value_container: &mut ValueContainer,
    ) -> Result<&mut Self, ()>
    where
        Self: Sized,
    {
        Err(())
    }
}
