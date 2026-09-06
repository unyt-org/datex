use crate::preludes::derive::DatexNative;
use core::{any::Any, time::Duration};

impl DatexNative for Duration {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
