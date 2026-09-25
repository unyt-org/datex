use core::cell::RefCell;

use crate::{
    preludes::derive::{
        SharedReferencesCache, UpdateCallbackDataAccess, ValueContainer,
    },
    value_updates::{
        errors::UpdateError,
        update_data::*,
        update_handler::{UpdateCallbackData, UpdateHandlerImpl},
    },
    values::core_values::native::NativeCoreValue,
};
impl UpdateCallbackDataAccess for NativeCoreValue {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData> {
        self.value.get_update_callback_data()
    }
}

impl UpdateHandlerImpl for NativeCoreValue {
    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        self.value.try_append_entry(data, cache)
    }
    fn try_clear(
        &mut self,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<ValueContainer, UpdateError> {
        self.value.try_clear(cache)
    }
    fn try_decrement(
        &mut self,
        data: DecrementUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        self.value.try_decrement(data, cache)
    }
    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<ValueContainer, UpdateError> {
        self.value.try_delete_entry(data, cache)
    }
    fn try_increment(
        &mut self,
        data: IncrementUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        self.value.try_increment(data, cache)
    }
    fn try_list_splice(
        &mut self,
        data: ListSpliceUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        self.value.try_list_splice(data, cache)
    }
    fn try_replace(
        &mut self,
        data: ReplaceUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        self.value.try_replace(data, cache)
    }
    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        cache: &RefCell<SharedReferencesCache>,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        self.value.try_set_entry(data, cache)
    }
}
