use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::{AccessError, IndexOutOfBoundsError},
    traits::value_access::ValueAccess,
    utils::goat::Goat,
    values::{
        borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
        core_values::native::DatexNative,
        value::borrowed_value::{BorrowedCoreValue, BorrowedValue},
        value_container::value_key::BorrowedValueKey,
    },
};

impl<T> ValueAccess for Vec<T>
where
    T: DatexNative,
{
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
        cache: &mut SharedReferencesCache,
    ) -> Result<BorrowedValueContainer<'_>, AccessError> {
        match key {
            BorrowedValueKey::Index(index) => {
                if let Some(value) = self.get(index as usize) {
                    Ok(BorrowedValue {
                        inner: BorrowedCoreValue::Native(Goat::Borrowed(value)),
                        custom_type: Some(
                            self.value_datex_type(cache).convert_to_definition(),
                        ),
                    }
                    .into())
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
