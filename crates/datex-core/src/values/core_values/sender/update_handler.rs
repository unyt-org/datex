use crate::{
    value_updates::update_handler::{
        UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
    values::core_values::{native::DatexNativeBase, sender::Sender},
};

impl<T: DatexNativeBase + 'static> UpdateCallbackDataAccess for Sender<T> {}
impl<T: DatexNativeBase + 'static> UpdateHandlerImpl for Sender<T> {}
