use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    value_updates::{
        errors::UpdateError,
        update_data::AppendEntryUpdateData,
        update_handler::{UpdateCallbackDataAccess, UpdateHandlerImpl},
    },
    values::core_values::{native::DatexNativeBase, receiver::Receiver},
};
use core::cell::RefCell;

impl<T: DatexNativeBase + 'static> UpdateCallbackDataAccess for Receiver<T> {}
impl<T: DatexNativeBase + 'static> UpdateHandlerImpl for Receiver<T> {
    fn try_append_entry(
        &mut self,
        data: AppendEntryUpdateData,
        _cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        if let Receiver::RemoteSender { sender, .. } = self {
            sender
                .start_send(
                    T::try_convert_from_value_container(data.value)
                        .map_err(|_| UpdateError::InvalidUpdate)?,
                )
                .map_err(|_| UpdateError::InvalidUpdate)
        } else {
            Err(UpdateError::InvalidUpdate)
        }
    }
}
