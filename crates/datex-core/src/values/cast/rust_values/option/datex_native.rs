use core::any::Any;
use crate::traits::get_datex_type::GetDatexType;
use crate::types::entity_type::EntityType;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::values::core_values::native::DatexNative;

impl<T: DatexNative + GetDatexType> DatexNative
for Option<T>
{
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
