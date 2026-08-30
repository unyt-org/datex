use core::any::Any;
use crate::types::entity_type::EntityType;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::values::core_values::native::DatexNative;
use crate::values::value::Value;
use crate::values::value::value_classification::ValueClassification;

impl DatexNative for Value
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn entity_type(&self, cache: &mut SharedReferencesCache) -> Option<EntityType> {
        match &self.classification {
            ValueClassification::Entity(entity_type) => Some(entity_type.clone()),
            _ => None,
        }
    }
}
