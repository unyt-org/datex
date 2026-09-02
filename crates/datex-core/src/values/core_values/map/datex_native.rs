use core::any::Any;
use crate::preludes::derive::{DatexNative, SharedReferencesCache, Type};
use crate::traits::static_classification::StaticClassification;
use crate::values::core_values::map::Map;

impl DatexNative for Map {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}