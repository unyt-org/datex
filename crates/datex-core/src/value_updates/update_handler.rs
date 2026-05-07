use crate::{
    prelude::*,
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            ReplaceUpdateData, SetEntryUpdateData,
        },
    },
    values::value_container::ValueContainer,
};
use crate::shared_values::base_shared_value_container::observers::TransceiverId;

pub trait UpdateHandler {
    fn try_replace(
        &mut self,
        data: ReplaceUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError>;
    fn try_set_entry(
        &mut self,
        data: SetEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError>;
    fn try_delete_entry(
        &mut self,
        data: DeleteEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<ValueContainer, UpdateError>;
    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError>;
    fn try_clear(
        &mut self,
        source_id: TransceiverId,
    ) -> Result<(), UpdateError>;
    fn try_list_splice(
        &mut self,
        data: ListSpliceUpdateData,
        source_id: TransceiverId,
    ) -> Result<Vec<ValueContainer>, UpdateError>;
}
