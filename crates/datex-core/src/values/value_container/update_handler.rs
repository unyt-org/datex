use crate::{
    prelude::*,
    shared_values::{
        base_shared_value_container::observers::TransceiverId,
        traits::SharedContainerCommon,
    },
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            SetEntryUpdateData, Update, UpdateOperation,
        },
        update_handler::{UpdateHandler, UpdateResult},
    },
    values::value_container::{ValueContainer, value_key::ValueKey},
};

impl UpdateHandler for ValueContainer {
    fn try_handle_update(&mut self, update: Update) -> UpdateResult {
        match self {
            ValueContainer::Local(local) => local.try_handle_update(update),
            ValueContainer::Shared(shared) => shared.try_handle_update(update),
        }
    }
}
