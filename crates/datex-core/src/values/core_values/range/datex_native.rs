use crate::{
    preludes::derive::DatexNative,
    traits::local_child_path_resolver::LocalChildPathResolver,
    values::core_values::{native::DatexNativeOps, range::Range},
};
use core::any::Any;

impl DatexNative for Range {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl DatexNativeOps for Range {}
impl LocalChildPathResolver for Range {}
