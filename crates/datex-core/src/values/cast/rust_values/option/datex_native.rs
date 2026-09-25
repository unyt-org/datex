use crate::{
    traits::local_child_path_resolver::LocalChildPathResolver,
    values::core_values::native::{DatexNative, DatexNativeOps},
};
use core::any::Any;

impl<T: DatexNative + 'static> DatexNative for Option<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl<T: DatexNative + 'static> DatexNativeOps for Option<T> {}
impl<T: DatexNative + 'static> LocalChildPathResolver for Option<T> {}
