use crate::{
    traits::local_child_path_resolver::LocalChildPathResolver,
    values::core_values::{
        native::{DatexNative, DatexNativeBase, DatexNativeOps},
        sender::Sender,
    },
};
use core::any::Any;

impl<T: DatexNativeBase + 'static> DatexNative for Sender<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl<T: DatexNativeBase + 'static> DatexNativeOps for Sender<T> {}

impl<T: DatexNativeBase + 'static> LocalChildPathResolver for Sender<T> {}
