use crate::{
    prelude::*,
    shared_values::base_shared_value_container::observers::TransceiverId,
    value_updates::{
        UpdateReturn,
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DecrementUpdateData, DeleteEntryUpdateData,
            IncrementUpdateData, ListSpliceUpdateData, ReplaceUpdateData,
            SetEntryUpdateData, Update, UpdateData, UpdateOperation,
        },
    },
    values::value_container::{ValueContainer, value_key::ValueKey},
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
    fn try_handle_update(&mut self, update: Update) -> UpdateResult;

    fn try_set_entry(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: SetEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        Ok(self.try_handle_update(Update::new(
            source_id,
            UpdateData::new_with_path(UpdateOperation::SetEntry(Box::new(data)), path),
        ))?.try_into().expect("UpdateReturn should be convertible into Result<Option<ValueContainer>, UpdateError>"))
    }

    fn try_delete_entry(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: DeleteEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        Ok(self.try_handle_update(Update::new(
            source_id,
            UpdateData::new_with_path(UpdateOperation::DeleteEntry(Box::new(data)), path),
        ))?.try_into().expect("UpdateReturn should be convertible into Result<Option<ValueContainer>, UpdateError>"))
    }

    fn try_append_entry(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: AppendEntryUpdateData,
    ) -> Result<(), UpdateError> {
        Ok(self.try_handle_update(Update::new(
            source_id,
            UpdateData::new_with_path(UpdateOperation::AppendEntry(Box::new(data)), path),
        ))?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<(), UpdateError>",
        ))
    }

    fn try_clear(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        Ok(self.try_handle_update(Update::new(
            source_id,
            UpdateData::new_with_path(UpdateOperation::Clear, path),
        ))?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<ValueContainer, UpdateError>",
        ))
    }

    fn try_list_splice(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: ListSpliceUpdateData,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        Ok(self.try_handle_update(Update::new(
            source_id,
            UpdateData::new_with_path(UpdateOperation::ListSplice(Box::new(data)), path),
        ))?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<Vec<ValueContainer>, UpdateError>",
        ))
    }

    fn try_increment(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: IncrementUpdateData,
    ) -> Result<(), UpdateError> {
        Ok(self.try_handle_update(Update::new(
            source_id,
            UpdateData::new_with_path(UpdateOperation::Increment(Box::new(data)), path),
        ))?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<(), UpdateError>",
        ))
    }
    fn try_decrement(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: DecrementUpdateData,
    ) -> Result<(), UpdateError> {
        Ok(self.try_handle_update(Update::new(
            source_id,
            UpdateData::new_with_path(UpdateOperation::Decrement(Box::new(data)), path),
        ))?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<(), UpdateError>",
        ))
    }
    fn try_replace(
        &mut self,
        _path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: ReplaceUpdateData,
    ) -> Result<(), UpdateError> {
        Ok(self.try_handle_update(Update::new(
            source_id,
            UpdateData::new(UpdateOperation::Replace(Box::new(data))),
        ))?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<(), UpdateError>",
        ))
    }
}

pub trait UpdateHandlerImpl {
    /// Handles an update operation on the implementing type and returns an UpdateResult.
    /// The replace must be handled at a higher level, as it is not specific to the implementing type.
    fn try_update(&mut self, update: Update) -> UpdateResult {
        let (source_id, operation, path) = update.into_parts();
        match operation {
            UpdateOperation::SetEntry(box data) => {
                into_update_result(self.try_set_entry(path, source_id, data))
            }
            UpdateOperation::DeleteEntry(box data) => {
                into_update_result(self.try_delete_entry(path, source_id, data))
            }
            UpdateOperation::AppendEntry(box data) => {
                into_update_result(self.try_append_entry(path, source_id, data))
            }
            UpdateOperation::Clear => {
                into_update_result(self.try_clear(path, source_id))
            }
            UpdateOperation::ListSplice(box data) => {
                into_update_result(self.try_list_splice(path, source_id, data))
            }
            UpdateOperation::Increment(box data) => {
                into_update_result(self.try_increment(path, source_id, data))
            }
            UpdateOperation::Decrement(box data) => {
                into_update_result(self.try_decrement(path, source_id, data))
            }
            UpdateOperation::Replace(box _data) => unreachable!(
                "Replace operation should be handled at a higher level"
            ),
        }
    }

    fn try_set_entry(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        _data: SetEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_delete_entry(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        _data: DeleteEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_append_entry(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        _data: AppendEntryUpdateData,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_clear(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_list_splice(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        _data: ListSpliceUpdateData,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_increment(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        _data: IncrementUpdateData,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }
    fn try_decrement(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        _data: DecrementUpdateData,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }
}
