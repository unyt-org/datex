use crate::{
    shared_values::{ReferencedSharedContainer, traits::SharedContainerCommon},
    value_updates::{
        errors::UpdateError,
        update_data::Update,
        update_handler::{UpdateHandler, UpdateResult},
    },
};

/// Update implementation
/// Note: does not implement [UpdateHandler] directly, since we don't need a mutable reference to self
impl ReferencedSharedContainer {
    pub fn update(&self, update: Update) -> UpdateResult {
        if self.can_mutate() {
            self.base_shared_container_mut().update(update)
        } else {
            Err(UpdateError::ImmutableReference)
        }
    }
}
