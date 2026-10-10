use crate::{
    traits::local_child_path_resolver::LocalChildPathResolver,
    values::core_values::{
        endpoint::Endpoint,
        instant::Instant,
        native::{DatexNative, DatexNativeOps},
    },
};
use core::any::Any;

impl DatexNative for Instant {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl DatexNativeOps for Instant {}
impl LocalChildPathResolver for Instant {}

use crate::value_updates::update_handler::{
    UpdateCallbackDataAccess, UpdateHandlerImpl,
};

impl UpdateHandlerImpl for Instant {}
impl UpdateCallbackDataAccess for Instant {}
