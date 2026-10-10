use crate::{
    traits::{
        child_iterator::ChildIterator,
        local_child_path_resolver::LocalChildPathResolver,
    },
    values::core_values::{
        endpoint::Endpoint,
        native::{DatexNative, DatexNativeOps},
    },
};
use core::any::Any;

impl DatexNative for Endpoint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl DatexNativeOps for Endpoint {}
impl LocalChildPathResolver for Endpoint {}

use crate::value_updates::update_handler::{
    UpdateCallbackDataAccess, UpdateHandlerImpl,
};

impl UpdateHandlerImpl for Endpoint {}
impl UpdateCallbackDataAccess for Endpoint {}

impl ChildIterator for Endpoint {}
