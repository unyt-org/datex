use crate::{
    traits::datex_native_structural::DatexNativeStructural,
    types::r#type::Type,
    value_updates::update_handler::{
        UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
};
impl DatexNativeStructural for Type {}
impl UpdateCallbackDataAccess for Type {}
impl UpdateHandlerImpl for Type {}
