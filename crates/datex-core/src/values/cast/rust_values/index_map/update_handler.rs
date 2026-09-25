use crate::{
    preludes::derive::{
        AccessError::{self},
        ConvertCoreValue, KeyNotFoundError,
    },
    random::RandomState,
    values::core_values::map::MapAccessError,
};
use core::{cell::RefCell, hash::Hash, mem, ops::DerefMut};
use indexmap::IndexMap;

use crate::{
    preludes::derive::{
        BorrowedValueKey, ConvertValueContainer, SharedReferencesCache,
        ValueContainer,
    },
    types::error::TypeError,
    value_updates::{
        errors::UpdateError,
        update_data::{DeleteEntryUpdateData, SetEntryUpdateData},
        update_handler::{UpdateCallbackDataAccess, UpdateHandlerImpl},
    },
    values::core_values::native::DatexNativeBase,
};
impl<K, V> UpdateCallbackDataAccess for IndexMap<K, V, RandomState>
where
    K: ConvertValueContainer + Eq + Hash,
    V: ConvertValueContainer,
{
}

impl<K, V> UpdateHandlerImpl for IndexMap<K, V, RandomState>
where
    K: Eq + Hash + DatexNativeBase + 'static,
    V: DatexNativeBase + 'static,
{
    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        let key = data
            .key
            .into_value_container()
            .try_into_value()
            .map_err(|_| UpdateError::type_error(TypeError::Invalid))?;
        let value = data
            .value
            .try_into_value()
            .map_err(|_| UpdateError::type_error(TypeError::Invalid))?;
        Ok(self.insert(key, value).map(|previous| {
            previous.to_value_container(cache.borrow_mut().deref_mut())
        }))
    }

    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<ValueContainer, UpdateError> {
        let key = data
            .key
            .into_value_container()
            .try_into_value::<K>()
            .map_err(|_| UpdateError::type_error(TypeError::Invalid))?;

        self.shift_remove(&key)
            .map(|previous| {
                previous.to_value_container(cache.borrow_mut().deref_mut())
            })
            .ok_or_else(|| {
                AccessError::MapAccessError(MapAccessError::KeyNotFound(
                    KeyNotFoundError::new(
                        key.to_value_container(cache.borrow_mut().deref_mut()),
                    ),
                ))
                .into()
            })
    }

    fn try_clear(
        &mut self,
        _cache: &RefCell<SharedReferencesCache>,
    ) -> Result<ValueContainer, UpdateError> {
        let previous = core::mem::take(self);
        Ok(ValueContainer::Local(previous.to_core_value().into()))
    }
}
