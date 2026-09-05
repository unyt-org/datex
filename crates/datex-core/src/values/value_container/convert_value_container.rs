use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    values::{
        value_container::ValueContainer,
    },
};
use crate::preludes::derive::{BorrowedValueContainer, DatexNative, Goat, GoatMut};
use crate::traits::convert_value_container::ConvertValueContainer;
use crate::utils::sheep::Sheep;
use crate::utils::sheep_mut::SheepMut;

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

    fn try_from_value_container(value_container: ValueContainer) -> Result<Self, ValueContainer>
    where
        Self: Sized
    {
        Ok(value_container)
    }

    fn try_borrow_from_value_container(value_container: &ValueContainer) -> Result<&Self, ()>
    where
        Self: Sized
    {
        Ok(value_container)
    }

    fn try_borrow_mut_from_value_container(value_container: &mut ValueContainer) -> Result<&mut Self, ()>
    where
        Self: Sized
    {
        Ok(value_container)
    }
}
