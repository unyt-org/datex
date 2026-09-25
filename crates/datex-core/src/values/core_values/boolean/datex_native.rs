use crate::{
    preludes::derive::{
        DatexNative, UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
    traits::local_child_path_resolver::LocalChildPathResolver,
    values::core_values::{boolean::Boolean, native::DatexNativeOps},
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
impl UpdateHandlerImpl for Boolean {}
impl UpdateCallbackDataAccess for Boolean {}
