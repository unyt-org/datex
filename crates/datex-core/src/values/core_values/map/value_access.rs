use crate::{
    shared_values::errors::AccessError,
    traits::value_access::ValueAccess,
    values::{
        core_values::map::Map,
        value_container::{
            ValueContainer,
            borrowed_value_container::{
                BorrowedValueContainer, BorrowedValueContainerMut,
            },
        },
    },
};

impl ValueAccess for Map {
    fn try_get_property(
        &self,
        key: &ValueContainer,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        Ok(self.try_get(key)?.into())
    }

    fn try_get_property_mut(
        &mut self,
        key: &ValueContainer,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        self.try_get_mut(key)?
            .try_into()
            .map_err(|_| AccessError::InvalidIndexKey)
    }
}
