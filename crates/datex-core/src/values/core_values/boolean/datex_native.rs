use crate::{
    traits::local_child_path_resolver::LocalChildPathResolver,
    value_updates::update_handler::{
        UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
    values::core_values::{
        boolean::Boolean,
        native::{DatexNative, DatexNativeOps},
    },
};
use core::any::Any;

impl DatexNative for Boolean {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl DatexNativeOps for Boolean {}
impl LocalChildPathResolver for Boolean {}
use crate::traits::iter_parts::IterParts;
impl IterParts for Boolean {}
impl UpdateHandlerImpl for Boolean {}
impl UpdateCallbackDataAccess for Boolean {}
