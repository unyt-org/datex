use crate::{preludes::derive::DatexNative, values::core_values::map::Map};
use core::any::Any;

impl DatexNative for Map {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
