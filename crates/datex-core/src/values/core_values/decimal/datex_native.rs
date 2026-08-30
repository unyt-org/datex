use core::any::Any;
use crate::preludes::derive::{DatexNative};
use crate::values::core_values::decimal::Decimal;

impl DatexNative for Decimal {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}