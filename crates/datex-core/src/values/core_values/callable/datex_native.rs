use core::any::Any;
use crate::preludes::derive::{DatexNative};
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::callable::Callable;

impl DatexNative for Callable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}