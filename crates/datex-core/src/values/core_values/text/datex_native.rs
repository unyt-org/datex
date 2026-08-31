use core::any::Any;
use crate::preludes::derive::{DatexNative, Type};
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::text::Text;

impl DatexNative for Text {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}