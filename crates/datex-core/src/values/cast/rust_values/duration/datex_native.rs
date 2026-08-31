use core::any::Any;
use core::time::Duration;
use crate::preludes::derive::DatexNative;

impl DatexNative for Duration {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
