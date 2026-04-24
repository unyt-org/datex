use crate::{
    prelude::*,
    values::{
        core_values::map::Map,
        value_container::{ValueContainer, value_key::BorrowedValueKey},
    },
};

use crate::{
    shared_values::observers::TransceiverId,
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData,
        },
        update_handler::UpdateHandler,
    },
};
use core::result::Result;

impl UpdateHandler for Map {
    fn try_replace(
        &mut self,
        _data: ReplaceUpdateData,
        _source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        todo!()
    }

    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        _source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        let key = BorrowedValueKey::from(data.key);
        self.try_set(key, data.value)
            .map_err(|err| UpdateError::AccessError(err.into()))
    }

    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        _source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        let key = BorrowedValueKey::from(data.key);
        self.try_delete(key)
            .map_err(|err| UpdateError::AccessError(err.into()))
    }

    fn try_append_entry(
        &mut self,
        _data: AppendEntryUpdateData,
        _source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_clear(
        &mut self,
        _source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        self.try_clear_inner()
            .map_err(|err| UpdateError::AccessError(err.into()))
    }

    fn try_list_splice(
        &mut self,
        _data: ListSpliceUpdateData,
        _source_id: TransceiverId,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }
}
