use crate::value_updates::errors::UpdateError;
use crate::value_updates::update_data::{DeleteEntryUpdateData, ListSpliceUpdateData, ReplaceUpdateData, SetEntryUpdateData};
use crate::values::value_container::ValueContainer;

pub trait UpdateHandler {
    fn try_replace(&self, data: ReplaceUpdateData) -> Result<ValueContainer, UpdateError>;
    fn try_set_entry(&self, data: SetEntryUpdateData) -> Result<(), UpdateError>;
    fn try_delete_entry(&self, data: DeleteEntryUpdateData) -> Result<ValueContainer, UpdateError>;
    fn try_clear(&self) -> Result<Vec<ValueContainer>, UpdateError>;
    fn try_list_splice(&self, data: ListSpliceUpdateData) -> Result<Vec<ValueContainer>, UpdateError>;
}