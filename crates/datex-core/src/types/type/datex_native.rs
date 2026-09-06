use crate::{
    preludes::derive::DatexNative,
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    types::r#type::Type,
};
use core::any::Any;

impl DatexNative for Type {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Classification for Type {}

impl StaticClassification for Type {}
