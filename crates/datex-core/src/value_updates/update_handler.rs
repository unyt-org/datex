use crate::{
    prelude::*,
    shared_values::base_shared_value_container::observers::{
        ObserverCallback, TransceiverId,
    },
    value_updates::{
        UpdateReturn,
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DecrementUpdateData, DeleteEntryUpdateData,
            IncrementUpdateData, ListSpliceUpdateData, ReplaceUpdateData,
            SetEntryUpdateData, Update, UpdateData, UpdateOperation,
        },
    },
    values::{
        value::Value,
        value_container::{ValueContainer, value_key::ValueKey},
    },
};
use core::fmt::{Debug, Formatter};

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
        Ok(self.try_handle_update(
            Update::new(
                source_id,
                UpdateData::new_with_path(
                    UpdateOperation::SetEntry(Box::new(data)),
                    path,
                ),
            )
        )?.try_into().expect("UpdateReturn should be convertible into Result<Option<ValueContainer>, UpdateError>"))
    }

    fn try_delete_entry(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: DeleteEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        Ok(self.try_handle_update(
            Update::new(
                source_id,
                UpdateData::new_with_path(
                    UpdateOperation::DeleteEntry(Box::new(data)),
                    path,
                ),
            )
        )?.try_into().expect("UpdateReturn should be convertible into Result<Option<ValueContainer>, UpdateError>"))
    }

    fn try_append_entry(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: AppendEntryUpdateData,
    ) -> Result<(), UpdateError> {
        let _: () = self.try_handle_update(
            Update::new(
                source_id,
                UpdateData::new_with_path(
                    UpdateOperation::AppendEntry(Box::new(data)),
                    path,
                ),
            )
        )?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<(), UpdateError>",
        );
        Ok(())
    }

    fn try_clear(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        Ok(self.try_handle_update(
            Update::new(
                source_id,
                UpdateData::new_with_path(UpdateOperation::Clear, path),
            )
        )?
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
        Ok(self.try_handle_update(
            Update::new(
                source_id,
                UpdateData::new_with_path(
                    UpdateOperation::ListSplice(Box::new(data)),
                    path,
                ),
            )
        )?
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
        let _: () = self.try_handle_update(
            Update::new(
                source_id,
                UpdateData::new_with_path(
                    UpdateOperation::Increment(Box::new(data)),
                    path,
                ),
            )
        )?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<(), UpdateError>",
        );
        Ok(())
    }
    fn try_decrement(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: DecrementUpdateData,
    ) -> Result<(), UpdateError> {
        let _: () = self.try_handle_update(
            Update::new(
                source_id,
                UpdateData::new_with_path(
                    UpdateOperation::Decrement(Box::new(data)),
                    path,
                ),
            )
        )?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<(), UpdateError>",
        );
        Ok(())
    }
    fn try_replace(
        &mut self,
        path: Vec<ValueKey>,
        source_id: TransceiverId,
        data: ReplaceUpdateData,
    ) -> Result<(), UpdateError> {
        let _: () = self.try_handle_update(
            Update::new(
                source_id,
                UpdateData::new_with_path(
                    UpdateOperation::Replace(Box::new(data)),
                    path,
                ),
            )
        )?
        .try_into()
        .expect(
            "UpdateReturn should be convertible into Result<(), UpdateError>",
        );
        Ok(())
    }
}

/// The local observer callback hold the callback and the path of the value if referenced
/// by a shared container
pub struct UpdateCallbackData {
    pub callback: ObserverCallback,
    pub path: Vec<ValueKey>,
}
impl UpdateCallbackData {
    /// Creates a new UpdateCallbackData with the same callback,
    /// appending the provided child_key to the path.
    pub fn with_child_path(&self, child_key: impl Into<ValueKey>) -> Self {
        let mut new_path = self.path.clone();
        new_path.push(child_key.into());
        UpdateCallbackData {
            callback: self.callback.clone(),
            path: new_path,
        }
    }
}

impl Debug for UpdateCallbackData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LocalObserveData")
            .field("callback", &"<ObserverCallback>")
            .field("path", &self.path)
            .finish()
    }
}

pub trait UpdateCallbackDataAccess {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData>;
}

pub trait InternalMutabilityUpdateHandler: UpdateCallbackDataAccess {
    fn set_update_callback_data(
        &mut self,
        observe_data: Option<UpdateCallbackData>,
    );

    /// Updates the update callback data for a child value based on the current update callback data of the parent.
    fn set_child_update_callback_data(
        &self,
        child_key: impl Into<ValueKey>,
        child_value: &mut Value,
    ) {
        if let Some(callback_data) = self.get_update_callback_data() {
            let child_callback_data = callback_data.with_child_path(child_key);
            child_value.set_update_callback_data(Some(child_callback_data));
        } else {
            child_value.set_update_callback_data(None);
        }
    }

    /// Updates the update callback data for a child value if it is a [ValueContainer::Local],
    /// based on the current update callback data of the parent.
    fn set_child_update_callback_data_if_local(
        &self,
        child_key: &(impl Into<ValueKey> + Clone),
        child_value: &mut ValueContainer,
    ) {
        if let ValueContainer::Local(child_value) = child_value {
            self.set_child_update_callback_data(child_key.clone(), child_value);
        }
    }

    /// Triggers the update callback if the source_id is provided and the callback data is available.
    fn maybe_trigger_update_callback(
        &self,
        source_id: Option<TransceiverId>,
        update_operation_generator: impl FnOnce() -> UpdateOperation,
    ) {
        if let Some(source_id) = source_id
            && let Some(callback_data) = self.get_update_callback_data()
        {
            let operation = update_operation_generator();
            (callback_data.callback)(&Update::new(
                source_id,
                UpdateData::new_with_path(
                    operation,
                    callback_data.path.clone(),
                ),
            ));
        }
    }
}

pub trait UpdateHandlerImpl: UpdateCallbackDataAccess {
    /// Handles an update operation on the implementing type and returns an UpdateResult.
    /// The replacement operation must be handled at a higher level, as it is not specific to the implementing type.
    /// If the optional source_id is provided, it should be used to notify observers of the internal update.
    fn try_update(
        &mut self,
        operation: UpdateOperation,
        source_id: Option<TransceiverId>,
    ) -> UpdateResult {
        let maybe_callback_data = if let Some(callback_data) =
            self.get_update_callback_data()
            && let Some(source_id) = &source_id
        {
            Some((
                Update::new(
                    source_id.clone(),
                    UpdateData::new_with_path(
                        operation.clone(),
                        callback_data.path.clone(),
                    ),
                ),
                callback_data.callback.clone(),
            ))
        } else {
            None
        };

        let ret = match operation {
            UpdateOperation::SetEntry(data) => {
                into_update_result(self.try_set_entry(*data))
            }
            UpdateOperation::DeleteEntry(data) => {
                into_update_result(self.try_delete_entry(*data))
            }
            UpdateOperation::AppendEntry(data) => {
                into_update_result(self.try_append_entry(*data))
            }
            UpdateOperation::Clear => into_update_result(self.try_clear()),
            UpdateOperation::ListSplice(data) => {
                into_update_result(self.try_list_splice(*data))
            }
            UpdateOperation::Increment(data) => {
                into_update_result(self.try_increment(*data))
            }
            UpdateOperation::Decrement(data) => {
                into_update_result(self.try_decrement(*data))
            }
            UpdateOperation::Replace(_data) => {
                Err(UpdateError::InvalidUpdate)
            }
        }?;

        // trigger callback
        if let Some((update, callback)) = maybe_callback_data {
            callback(&update);
        }

        Ok(ret)
    }

    fn try_set_entry(
        &mut self,
        _data: SetEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_delete_entry(
        &mut self,
        _data: DeleteEntryUpdateData,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_append_entry(
        &mut self,
        _data: AppendEntryUpdateData,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_clear(&mut self) -> Result<ValueContainer, UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_list_splice(
        &mut self,
        _data: ListSpliceUpdateData,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }

    fn try_increment(
        &mut self,
        _data: IncrementUpdateData,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }
    fn try_decrement(
        &mut self,
        _data: DecrementUpdateData,
    ) -> Result<(), UpdateError> {
        Err(UpdateError::InvalidUpdate)
    }
}
