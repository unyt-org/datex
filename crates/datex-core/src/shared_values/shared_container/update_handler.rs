use crate::{
    shared_values::{
        SharedContainer, base_shared_value_container::observers::TransceiverId,
        traits::SharedContainerCommon,
    },
    value_updates::{
        UpdateReturn,
        errors::UpdateError,
        update_data::{SetEntryUpdateData, Update, UpdateData},
        update_handler::{UpdateHandler, UpdateResult},
    },
    values::value_container::ValueContainer,
};

/// Update implementation
/// Note: does not implement [UpdateHandler] directly, since we don't need a mutable reference to self
impl SharedContainer {
    pub fn update(&self, update: Update) -> UpdateResult {
        if let SharedContainer::Referenced(referenced) = self
            && !referenced.can_mutate()
        {
            return Err(UpdateError::ImmutableReference);
        }

        let observers = self
            .base_shared_container()
            .get_current_observers(&update.source_id);
        let update_clone = update.clone();
        let result = self.base_shared_container_mut().handle_update(update)?;
        for observer in observers {
            observer(&update_clone);
        }
        Ok(result)
    }

    // TODO: better way than duplicate implementation of those methods?
    pub fn try_set_entry(
        &self,
        data: SetEntryUpdateData,
        source_id: TransceiverId,
    ) -> Result<Option<ValueContainer>, UpdateError> {
        match self.update(UpdateData::SetEntry(data).with_source(source_id))? {
            UpdateReturn::SingleValue(value) => Ok(Some(value)),
            UpdateReturn::None => Ok(None),
            _ => unreachable!(),
        }
    }
}
