use core::cell::RefCell;
use crate::{
    value_updates::update_handler::{
        UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
    values::core_values::{native::DatexNativeBase, receiver::Receiver},
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::value_updates::errors::UpdateError;
use crate::value_updates::update_data::AppendEntryUpdateData;

impl<T: DatexNativeBase + 'static> UpdateCallbackDataAccess for Receiver<T> {}
impl<T: DatexNativeBase + 'static> UpdateHandlerImpl for Receiver<T> {
    fn try_append_entry(&mut self, data: AppendEntryUpdateData, _cache: &RefCell<SharedReferencesCache>) -> Result<(), UpdateError> {
        if let Receiver::RemoteSender { sender, ..  } = self {
            sender
                .start_send(T::try_convert_from_value_container(data.value).map_err(|_| UpdateError::InvalidUpdate)?)
                .map_err(|_| UpdateError::InvalidUpdate)
        } else {
            Err(UpdateError::InvalidUpdate)
        }

    }
}
