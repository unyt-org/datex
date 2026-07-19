use crate::{
    shared_values::{
        SharedContainer, base_shared_value_container::observers::TransceiverId,
        traits::SharedContainerCommon,
    },
    value_updates::{
        UpdateReturn,
        errors::UpdateError,
        update_data::{
            SetEntryUpdateData, Update, UpdateData, UpdateOperation,
        },
        update_handler::{UpdateHandler, UpdateResult},
    },
    values::value_container::{ValueContainer, value_key::ValueKey},
};

impl UpdateHandler for SharedContainer {
    fn try_handle_update(&mut self, update: Update) -> UpdateResult {
        if let SharedContainer::Referenced(referenced) = self
            && !referenced.can_mutate()
        {
            return Err(UpdateError::ImmutableReference);
        }

        let update_clone = update.clone();
        let (source_id, operation, path) = update.into_parts();

        let observers = self
            .base_shared_container()
            .get_current_observers(&source_id);

        let result = self
            .base_shared_container_mut()
            .try_handle_update(operation, path)?;

        // call observers
        for observer in observers {
            observer(&update_clone);
        }

        Ok(result)
    }
}
