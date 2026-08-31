use core::any::Any;
use crate::preludes::derive::{DatexNative};
use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::types::r#type::Type;

impl DatexNative for Type {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Classification for Type {}

impl HasClassification for Type {}