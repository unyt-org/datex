use crate::datex_proxy::DatexProxyType;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::shared_values::errors::{AccessError, IndexOutOfBoundsError};
use crate::traits::value_access::ValueAccess;
use crate::utils::goat::Goat;
use crate::values::borrowed_value_container::{BorrowedValueContainer, BorrowedValueContainerMut};
use crate::values::core_values::native::{DatexNative};
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedValue};
use crate::values::value_container::value_key::BorrowedValueKey;

impl<T> ValueAccess for Vec<T> where T: DatexNative + DatexProxyType {
    fn try_get_property(&self, key: BorrowedValueKey, cache: &mut SharedReferencesCache) -> Result<BorrowedValueContainer<'_>, AccessError> {
        match key {
            BorrowedValueKey::Index(index) => {
                if let Some(value) = self.get(index as usize) {
                    Ok(BorrowedValue {
                        inner: BorrowedCoreValue::Native(Goat::Borrowed(value)),
                        custom_type: Some(T::datex_type(cache).convert_to_definition()),
                    }.into())
                } else {
                    Err(AccessError::IndexOutOfBounds(IndexOutOfBoundsError { index: index as u32 }))
                }
            }
            _ => Err(AccessError::InvalidOperation(format!("Invalid key: {:?}", key))), // TODO: better access errors
        }
    }

    fn try_get_property_mut(&mut self, key: BorrowedValueKey, cache: &mut SharedReferencesCache) -> Result<BorrowedValueContainerMut<'_>, AccessError> {
        todo!()
    }
}