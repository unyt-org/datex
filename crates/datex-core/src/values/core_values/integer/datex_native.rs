use core::any::Any;
use crate::preludes::derive::{DatexNative, SharedReferencesCache};
use crate::values::core_values::integer::Integer;
use crate::values::value::value_classification::ValueClassification;

impl DatexNative for Integer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}