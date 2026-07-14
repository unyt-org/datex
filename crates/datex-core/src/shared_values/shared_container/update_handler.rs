use crate::{
    shared_values::SharedContainer,
    value_updates::{
        update_data::Update,
        update_handler::{UpdateHandler, UpdateResult},
    },
};

/// Update implementation
/// Note: does not implement [UpdateHandler] directly, since we don't need a mutable reference to self
impl SharedContainer {
    pub fn update(&self, update: Update) -> UpdateResult {
        match self {
            SharedContainer::Owned(owned) => owned.update(update),
            SharedContainer::Referenced(referenced) => {
                referenced.update(update)
            }
        }
    }
}
