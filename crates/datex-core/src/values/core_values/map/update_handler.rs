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
    shared_values::base_shared_value_container::observers::TransceiverId,
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            SetEntryUpdateData,
        },
        update_handler::{UpdateCallbackData, UpdateHandlerImpl},
    },
};
use core::result::Result;
use crate::value_updates::update_handler::InternalMutabilityUpdateHandler;

impl InternalMutabilityUpdateHandler for Map {
    fn set_update_callback_data(
        &mut self,
        observe_data: Option<UpdateCallbackData>,
    ) {
        self.update_callback_data = observe_data;
    }
}


impl UpdateHandlerImpl for Map {
    fn try_set_entry(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        data: SetEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        let key = BorrowedValueKey::from(data.key);
        self.try_set(key, data.value)
            .map_err(UpdateError::access_error)
    }

    fn try_delete_entry(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        data: DeleteEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        let key = BorrowedValueKey::from(data.key);
        self.try_delete(key)
            .map_err(UpdateError::access_error)
            .map(Some)
    }

    fn try_clear(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        self.try_clear_inner().map_err(UpdateError::access_error)
    }
}
