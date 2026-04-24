use crate::{
    prelude::*,
    shared_values::errors::AccessError,
    values::{
        core_values::list::List,
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

impl UpdateHandler for List {
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
        let key = BorrowedValueKey::from(data.key).try_as_index().ok_or_else(
            || UpdateError::AccessError(AccessError::InvalidIndexKey),
        )?;
        self.try_set(key, data.value)
            .map(|_| ())
            .map_err(|err| UpdateError::AccessError(err.into()))
    }

    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        _source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        let key = BorrowedValueKey::from(data.key).try_as_index().ok_or_else(
            || UpdateError::AccessError(AccessError::InvalidIndexKey),
        )?;
        self.try_delete(key)
            .map_err(|err| UpdateError::AccessError(err.into()))
    }

    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        self.push(data.value);
        Ok(())
    }

    fn try_clear(
        &mut self,
        _source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        self.clear();
        Ok(())
    }

    fn try_list_splice(
        &mut self,
        data: ListSpliceUpdateData,
        _source_id: TransceiverId,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        Ok(self.splice(data.start..data.delete_count, data.items))
    }
}
