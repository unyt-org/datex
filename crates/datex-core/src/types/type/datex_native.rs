use crate::{
    traits::{
        child_iterator::ChildIterator, classification::Classification,
        local_child_path_resolver::LocalChildPathResolver,
    },
    types::r#type::Type,
    values::core_values::native::{DatexNative, DatexNativeOps},
};

use core::any::Any;

impl DatexNative for Type {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Classification for Type {}

impl DatexNativeOps for Type {}
impl LocalChildPathResolver for Type {}
impl ChildIterator for Type {}
