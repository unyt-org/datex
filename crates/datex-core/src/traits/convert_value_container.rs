use crate::preludes::derive::{SharedReferencesCache, ValueContainer};
use crate::values::borrowed_value_container::BorrowedValueContainer;

pub trait ConvertValueContainer {
    fn to_value_container(
        self,
        cache: &mut SharedReferencesCache,
    ) -> ValueContainer;

    fn as_borrowed_value_container(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> BorrowedValueContainer;

    fn try_from_value_container(
        value_container: ValueContainer,
    ) -> Result<Self, ValueContainer>
    where
        Self: Sized;

    fn try_borrow_from_value_container(
        value_container: &ValueContainer,
    ) -> Result<&Self, ()>
    where
        Self: Sized;

    fn try_borrow_mut_from_value_container(
        value_container: &mut ValueContainer,
    ) -> Result<&mut Self, ()>
    where
        Self: Sized;
}
