use core::any::Any;
use crate::preludes::derive::{CoreValue, DatexNative};
use crate::values::core_values::native::DatexNativeBase;

impl<T: DatexNativeBase + 'static> DatexNative for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}