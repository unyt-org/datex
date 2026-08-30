use core::any::Any;
use crate::preludes::derive::{DatexNative, SharedReferencesCache};
use crate::types::entity_type::EntityType;
use crate::values::core_values::callable::Callable;
use crate::values::value::value_classification::ValueClassification;

impl DatexNative for Callable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}