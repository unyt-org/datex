use core::{cell::RefCell, mem, ops::DerefMut};

use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::errors::{AccessError, IndexOutOfBoundsError},
    types::error::TypeError,
    value_updates::{
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DeleteEntryUpdateData, ListSpliceUpdateData,
            SetEntryUpdateData,
        },
        update_handler::{UpdateCallbackDataAccess, UpdateHandlerImpl},
    },
    values::{
        core_values::native::DatexNativeBase,
        value_container::{ValueContainer},
    },
};
use crate::values::core_values::sender::Sender;

impl<T: DatexNativeBase + 'static> UpdateCallbackDataAccess for Sender<T> {}
impl<T: DatexNativeBase + 'static> UpdateHandlerImpl for Sender<T> {
}
