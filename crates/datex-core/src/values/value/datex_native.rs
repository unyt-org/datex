use core::any::Any;
use crate::values::core_values::native::DatexNative;
use crate::values::value::Value;

impl DatexNative for Value
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
