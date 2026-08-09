use crate::{
    value_updates::{
        update_data::Update,
        update_handler::{UpdateHandler, UpdateResult},
    },
    values::value_container::ValueContainer,
};

impl UpdateHandler for ValueContainer {
    fn try_handle_update(&mut self, update: Update) -> UpdateResult {
        match self {
            ValueContainer::Local(local) => {
                let (source_id, operation, path) = update.into_parts();

                local.try_update_collapsed_local_inner(
                    operation,
                    path,
                    Some(source_id),
                )
            }
            ValueContainer::Shared(shared) => shared.try_handle_update(update),
        }
    }
}
