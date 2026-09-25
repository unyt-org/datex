use core::cell::RefCell;

use crate::{
    preludes::derive::{SharedReferencesCache, ValueContainer},
    value_updates::{
        errors::UpdateError,
        update_data::*,
        update_handler::{UpdateCallbackDataAccess, UpdateHandlerImpl},
    },
    values::core_values::native::DatexNative,
};

impl<T: DatexNative> UpdateCallbackDataAccess for Box<T> {}
impl<T: DatexNative> UpdateHandlerImpl for Box<T> {
    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        (**self).try_append_entry(data, cache)
    }
    fn try_clear(
        &mut self,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<ValueContainer, UpdateError> {
        (**self).try_clear(cache)
    }
    fn try_decrement(
        &mut self,
        data: DecrementUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        (**self).try_decrement(data, cache)
    }
    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        (**self).try_delete_entry(data, cache)
    }
    fn try_increment(
        &mut self,
        data: IncrementUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        (**self).try_increment(data, cache)
    }
    fn try_list_splice(
        &mut self,
        data: ListSpliceUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        (**self).try_list_splice(data, cache)
    }
    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        (**self).try_set_entry(data, cache)
    }
}
