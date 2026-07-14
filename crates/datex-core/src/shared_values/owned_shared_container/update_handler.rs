use crate::shared_values::{OwnedSharedContainer};
use crate::shared_values::traits::SharedContainerCommon;
use crate::value_updates::update_data::Update;
use crate::value_updates::update_handler::{UpdateHandler, UpdateResult};

/// Update implementation
/// Note: does not implement [UpdateHandler] directly, since we don't need a mutable reference to self
impl OwnedSharedContainer {
    pub fn update(&self, update: Update) -> UpdateResult {
        self.base_shared_container_mut().update(update)
    }
}