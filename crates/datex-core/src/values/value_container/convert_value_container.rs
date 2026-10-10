use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::convert_value_container::ConvertValueContainer,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        value_container::ValueContainer,
    },
};

impl ConvertValueContainer for ValueContainer {
    fn to_value_container(self) -> ValueContainer {
        self
    }

    fn as_borrowed_value_container(&self) -> BorrowedValueContainer<'_> {
        BorrowedValueContainer::from(self)
    }
    fn as_borrowed_value_container_mut(
        &mut self,
    ) -> BorrowedValueContainerMut<'_> {
        BorrowedValueContainerMut::from(self)
    }

    fn try_from_value_container(
        value_container: ValueContainer,
    ) -> Result<Self, ValueContainer>
    where
        Self: Sized,
    {
        Ok(value_container)
    }

    fn try_borrow_from_value_container(
        value_container: &ValueContainer,
    ) -> Result<&Self, ()>
    where
        Self: Sized,
    {
        Ok(value_container)
    }

    fn try_borrow_mut_from_value_container(
        value_container: &mut ValueContainer,
    ) -> Result<&mut Self, ()>
    where
        Self: Sized,
    {
        Ok(value_container)
    }
}
