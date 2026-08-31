use core::any::Any;
use crate::preludes::derive::{DatexNative};
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

impl DatexNative for TypedDecimal {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}