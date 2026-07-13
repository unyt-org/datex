use crate::{
    prelude::*,
    shared_values::base_shared_value_container::observers::TransceiverId,
    value_updates::{
        UpdateReturn,
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData, Update, UpdateData,
        },
    },
    values::value_container::ValueContainer,
};

pub type UpdateResult = Result<UpdateReturn, UpdateError>;

/// Converts a Result with any types that can be converted into UpdateReturn and UpdateError into an UpdateResult.
pub fn into_update_result<T: Into<UpdateReturn>, E: Into<UpdateError>>(
    result: Result<T, E>,
) -> UpdateResult {
    match result {
        Ok(value) => Ok(value.into()),
        Err(err) => Err(err.into()),
    }
}

pub trait UpdateHandler {
    fn update(&mut self, update: Update) -> UpdateResult {
        self.update_inner(update)
    }

    fn update_inner(&mut self, update: Update) -> UpdateResult {
        match update.data {
            UpdateData::AppendEntry(data) => into_update_result(
                self.try_append_entry(data, update.source_id),
            ),
            UpdateData::Clear => {
                into_update_result(self.try_clear(update.source_id))
            }
            UpdateData::Replace(data) => {
                into_update_result(self.try_replace(data, update.source_id))
            }
            UpdateData::SetEntry(data) => {
                into_update_result(self.try_set_entry(data, update.source_id))
            }
            UpdateData::DeleteEntry(data) => into_update_result(
                self.try_delete_entry(data, update.source_id),
            ),
            UpdateData::ListSplice(data) => {
                into_update_result(self.try_list_splice(data, update.source_id))
            }
        }
    }

    fn try_replace(
        &mut self,
        _data: ReplaceUpdateData,
        _source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        unimplemented!()
    }

    fn try_set_entry(
        &mut self,
        _data: SetEntryUpdateData,
        _source_id: TransceiverId,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        unimplemented!()
    }

    fn try_delete_entry(
        &mut self,
        _data: DeleteEntryUpdateData,
        _source_id: TransceiverId,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        unimplemented!()
    }

    fn try_append_entry(
        &mut self,
        _data: AppendEntryUpdateData,
        _source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        unimplemented!()
    }

    fn try_clear(
        &mut self,
        _source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        unimplemented!()
    }

    fn try_list_splice(
        &mut self,
        _data: ListSpliceUpdateData,
        _source_id: TransceiverId,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        unimplemented!()
    }
}
