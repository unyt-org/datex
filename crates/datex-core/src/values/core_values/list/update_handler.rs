use crate::{
    prelude::*,
    shared_values::{
        base_shared_value_container::observers::TransceiverId,
        errors::AccessError,
    },
    value_updates::update_data::{DecrementUpdateData, IncrementUpdateData},
    values::{
        core_values::list::List,
        value_container::{
            ValueContainer,
            value_key::{BorrowedValueKey, ValueKey},
        },
    },
};

use crate::value_updates::{
    errors::UpdateError,
    update_data::{
        AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
        SetEntryUpdateData,
    },
    update_handler::UpdateHandlerImpl,
};
use core::result::Result;

impl UpdateHandlerImpl for List {
    fn try_set_entry(
        &mut self,
        _path: Vec<ValueKey>,
        _transceiver_id: TransceiverId,
        data: SetEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        let key = BorrowedValueKey::from(data.key).try_as_index().ok_or_else(
            || UpdateError::access_error(AccessError::InvalidIndexKey),
        )?;
        self.try_set(key, data.value)
            .map(Some)
            .map_err(UpdateError::access_error)
    }

    fn try_delete_entry(
        &mut self,
        _path: Vec<ValueKey>,
        _transceiver_id: TransceiverId,
        data: DeleteEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        let key = BorrowedValueKey::from(data.key).try_as_index().ok_or_else(
            || UpdateError::access_error(AccessError::InvalidIndexKey),
        )?;
        self.try_delete(key)
            .map_err(UpdateError::access_error)
            .map(Some)
    }

    fn try_append_entry(
        &mut self,
        _path: Vec<ValueKey>,
        _transceiver_id: TransceiverId,
        data: AppendEntryUpdateData,
    ) -> Result<(), UpdateError> {
        self.push(data.value);
        Ok(())
    }

    fn try_clear(
        &mut self,
        _path: Vec<ValueKey>,
        _transceiver_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        let previous = core::mem::take(self);
        Ok(ValueContainer::Local(previous.into()))
    }

    fn try_list_splice(
        &mut self,
        _path: Vec<ValueKey>,
        _transceiver_id: TransceiverId,
        data: ListSpliceUpdateData,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        Ok(self
            .splice(data.start..(data.start + data.delete_count), data.items))
    }
    fn try_decrement(
        &mut self,
        _path: Vec<ValueKey>,
        _transceiver_id: TransceiverId,
        _data: DecrementUpdateData,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }
    fn try_increment(
        &mut self,
        _path: Vec<ValueKey>,
        _transceiver_id: TransceiverId,
        _data: IncrementUpdateData,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }
}
