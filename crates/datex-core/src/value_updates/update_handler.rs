use crate::shared_values::shared_containers::observers::TransceiverId;
use crate::value_updates::errors::UpdateError;
use crate::value_updates::update_data::{AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData, ReplaceUpdateData, SetEntryUpdateData};
use crate::values::value_container::ValueContainer;

pub trait UpdateHandler {
    fn try_replace(&self, data: ReplaceUpdateData, source_id: TransceiverId) -> Result<ValueContainer, UpdateError>;
    fn try_set_entry(&self, data: SetEntryUpdateData, source_id: TransceiverId) -> Result<(), UpdateError>;
    fn try_delete_entry(&self, data: DeleteEntryUpdateData, source_id: TransceiverId) -> Result<ValueContainer, UpdateError>;
    fn try_append_entry(&self, data: AppendEntryUpdateData, source_id: TransceiverId) -> Result<(), UpdateError>;
    fn try_clear(&self, source_id: TransceiverId) -> Result<Vec<ValueContainer>, UpdateError>;
    fn try_list_splice(&self, data: ListSpliceUpdateData, source_id: TransceiverId) -> Result<Vec<ValueContainer>, UpdateError>;
}