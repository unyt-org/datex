use crate::{
    value_updates::{
        errors::UpdateError,
        update_data::{Update, UpdateData, UpdateOperation},
        update_handler::{UpdateHandler, UpdateHandlerImpl, UpdateResult},
    },
    values::{core_value::CoreValue, value::Value},
};

impl UpdateHandler for Value {
    fn try_handle_update(&mut self, update: Update) -> UpdateResult {
        let (source_id, operation, path) = update.into_parts();
        match operation {
            UpdateOperation::Replace(_) => Err(UpdateError::InvalidUpdate),
            _ => {
                let update = Update::new(
                    source_id,
                    UpdateData::new_with_path(operation, path),
                );
                match &mut self.inner {
                    CoreValue::Map(map) => map.try_update(update),
                    CoreValue::List(list) => list.try_update(update),
                    _ => Err(UpdateError::InvalidUpdate),
                }
            }
        }
    }
}
