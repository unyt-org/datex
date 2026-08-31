use core::any::Any;
use crate::preludes::derive::{DatexNative};
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::boolean::Boolean;

impl DatexNative for Boolean {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}