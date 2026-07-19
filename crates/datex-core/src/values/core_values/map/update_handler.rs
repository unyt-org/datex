use crate::{
    prelude::*,
    values::{
        core_values::map::Map,
        value_container::{
            ValueContainer,
            value_key::{BorrowedValueKey, ValueKey},
        },
    },
};

use crate::{
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            SetEntryUpdateData,
        },
        update_handler::{
            InternalMutabilityUpdateHandler, UpdateCallbackData,
            UpdateCallbackDataAccess, UpdateHandlerImpl,
        },
    },
    values::core_values::map::MapKey,
};
use core::result::Result;

impl InternalMutabilityUpdateHandler for Map {
    fn set_update_callback_data(
        &mut self,
        observe_data: Option<UpdateCallbackData>,
    ) {
        // Update the update callback data for all child values
        for (key, child) in self.iter_local_values_mut() {
            child.set_update_callback_data(
                observe_data
                    .as_ref()
                    .map(|data| data.with_child_path(MapKey::from(key))),
            );
        }
        // Update the update callback data for the list itself
        self.update_callback_data = observe_data;
    }
}

impl UpdateCallbackDataAccess for Map {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData> {
        self.update_callback_data.as_ref()
    }
}

impl UpdateHandlerImpl for Map {
    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        let key = BorrowedValueKey::from(data.key);
        self.try_set_with_source(key, data.value, None)
            .map_err(UpdateError::access_error)
    }

    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        let key = BorrowedValueKey::from(data.key);
        self.try_delete_with_source(key, None)
            .map_err(UpdateError::access_error)
            .map(Some)
    }

    fn try_clear(&mut self) -> Result<ValueContainer, UpdateError> {
        self.try_clear_with_source(None)
            .map_err(UpdateError::access_error)
    }
}
