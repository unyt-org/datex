use core::any::Any;
use crate::values::core_values::native::{DatexNative, DatexNativeBase};

impl<T: DatexNativeBase + 'static> DatexNative
for Option<T>
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}