use crate::{
    traits::convert_value_container::ConvertValueContainer,
    values::value_container::{
        ValueContainer,
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
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
        // FIXME: Do we want to panic here if the conversion fails, or handle it differently?
        BorrowedValueContainerMut::try_from(self)
            .expect("Failed to get borrowed value container mut")
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
