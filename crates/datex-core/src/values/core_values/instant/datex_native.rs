use crate::{
    traits::{
        child_iterator::ChildIterator,
        local_child_path_resolver::LocalChildPathResolver,
    },
    value_updates::update_handler::{
        UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
    values::core_values::{
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

impl UpdateHandlerImpl for Instant {}
impl UpdateCallbackDataAccess for Instant {}

impl ChildIterator for Instant {}
