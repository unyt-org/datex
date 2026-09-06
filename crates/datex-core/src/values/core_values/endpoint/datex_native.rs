use crate::{
    preludes::derive::DatexNative, values::core_values::endpoint::Endpoint,
};
use core::any::Any;

impl DatexNative for Endpoint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
