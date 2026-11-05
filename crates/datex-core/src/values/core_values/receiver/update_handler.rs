use crate::{
    value_updates::update_handler::{
        UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
    values::core_values::{native::DatexNativeBase, receiver::Receiver},
};

impl<T: DatexNativeBase + 'static> UpdateCallbackDataAccess for Receiver<T> {}
impl<T: DatexNativeBase + 'static> UpdateHandlerImpl for Receiver<T> {}
