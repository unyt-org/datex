use crate::{
    traits::local_child_path_resolver::LocalChildPathResolver,
    values::core_values::native::{DatexNative, DatexNativeOps},
};
use core::{any::Any, time::Duration};

impl DatexNative for Duration {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
impl DatexNativeOps for Duration {}
impl LocalChildPathResolver for Duration { }
use crate::traits::child_iterator::ChildIterator;
impl ChildIterator for Duration { }
