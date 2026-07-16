use crate::{
    shared_values::{SharedContainer, traits::SharedContainerCommon},
    value_updates::{
        errors::UpdateError,
        update_data::Update,
        update_handler::{UpdateHandler, UpdateResult},
    },
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
}
