use crate::{
    traits::local_child_path_resolver::LocalChildPathResolver,
    value_updates::update_handler::{
        UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
    values::core_values::{
        callable::Callable,
        native::{DatexNative, DatexNativeOps},
    },
};

use core::any::Any;
impl DatexNative for Callable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl DatexNativeOps for Callable {}
impl LocalChildPathResolver for Callable {}
impl UpdateHandlerImpl for Callable {}
impl UpdateCallbackDataAccess for Callable {}
