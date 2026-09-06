use crate::values::{core_values::native::DatexNative, value::Value};
use core::any::Any;

impl DatexNative for Value {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
