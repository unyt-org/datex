use crate::{
    prelude::*,
    shared_values::errors::{AccessError, IndexOutOfBoundsError},
    traits::{get_datex_type::GetDatexType, value_access::ValueAccess},
    values::{
        core_values::native::DatexNativeBase,
        value_container::{
            ValueContainer,
            borrowed_value_container::{
                BorrowedValueContainer, BorrowedValueContainerMut,
            },
        },
    },
};

impl<T> ValueAccess for Vec<T>
where
    T: DatexNativeBase + GetDatexType,
{
    fn try_get_property(
        &self,
        key: &ValueContainer,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        match key.try_as::<u32>() {
            Some(index) => {
                if let Some(value) = self.get(*index as usize) {
                    Ok(value.as_borrowed_value_container())
                } else {
                    Err(AccessError::IndexOutOfBounds(IndexOutOfBoundsError {
                        index: *index,
                    }))
                }
            }
            _ => Err(AccessError::InvalidOperation("Invalid key".to_string())), // TODO: better access errors
        }
    }

    fn try_get_property_mut(
        &mut self,
        _key: &ValueContainer,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        todo!()
    }
}
