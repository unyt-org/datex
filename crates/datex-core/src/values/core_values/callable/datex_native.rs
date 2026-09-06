use crate::{
    preludes::derive::DatexNative, values::core_values::callable::Callable,
};
use core::any::Any;

impl DatexNative for Callable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
