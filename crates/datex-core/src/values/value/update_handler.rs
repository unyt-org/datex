use crate::{
    prelude::*,
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData,
        },
        update_handler::UpdateHandler,
    },
    values::{
        core_value::CoreValue, value::Value, value_container::ValueContainer,
    },
};
use crate::shared_values::base_shared_value_container::observers::TransceiverId;

impl UpdateHandler for Value {
    fn try_replace(
        &mut self,
        data: ReplaceUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => map.try_replace(data, source_id),
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => map.try_set_entry(data, source_id),
            CoreValue::List(ref mut list) => {
                list.try_set_entry(data, source_id)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => {
                map.try_delete_entry(data, source_id)
            }
            CoreValue::List(ref mut list) => {
                list.try_delete_entry(data, source_id)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => {
                map.try_append_entry(data, source_id)
            }
            CoreValue::List(ref mut list) => {
                list.try_append_entry(data, source_id)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_clear(
        &mut self,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError> {
        match self.inner {
            CoreValue::Map(ref mut map) => map.try_clear(source_id),
            _ => Err(UpdateError::InvalidUpdate),
        }
    }

    fn try_list_splice(
        &mut self,
        _data: ListSpliceUpdateData,
        _source_id: TransceiverId,
    ) -> Result<Vec<ValueContainer>, UpdateError> {
        match self.inner {
            CoreValue::List(ref mut list) => {
                list.try_list_splice(_data, _source_id)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
    }
}
