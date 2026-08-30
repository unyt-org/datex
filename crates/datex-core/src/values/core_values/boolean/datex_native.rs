use core::any::Any;
use crate::preludes::derive::{DatexNative, SharedReferencesCache};
use crate::types::entity_type::EntityType;
use crate::values::core_values::boolean::Boolean;

impl DatexNative for Boolean {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn entity_type(&self, cache: &mut SharedReferencesCache) -> Option<EntityType> {
        None
    }
}