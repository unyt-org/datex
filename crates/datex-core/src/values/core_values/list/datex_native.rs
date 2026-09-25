use crate::{
    preludes::derive::DatexNative,
    traits::local_child_path_resolver::LocalChildPathResolver,
    values::core_values::{list::List, native::DatexNativeOps},
};
use core::any::Any;

impl DatexNative for List {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl DatexNativeOps for List {}
