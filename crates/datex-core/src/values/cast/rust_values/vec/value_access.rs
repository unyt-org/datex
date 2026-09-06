use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::{AccessError, IndexOutOfBoundsError},
    traits::{get_datex_type::GetDatexType, value_access::ValueAccess},
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_values::native::DatexNativeBase,
        value_container::value_key::BorrowedValueKey,
    },
};

impl<T> ValueAccess for Vec<T>
where
    T: DatexNativeBase + GetDatexType,
{
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        match key {
            BorrowedValueKey::Index(index) => {
                if let Some(value) = self.get(index as usize) {
                    Ok(value.as_borrowed_value_container(cache))
                } else {
                    Err(AccessError::IndexOutOfBounds(IndexOutOfBoundsError {
                        index: index as u32,
                    }))
                }
            }
            _ => Err(AccessError::InvalidOperation(format!(
                "Invalid key: {:?}",
                key
            ))), // TODO: better access errors
        }
    }

    fn try_get_property_mut(
        &mut self,
        _key: BorrowedValueKey,
        _cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        todo!()
    }
}
