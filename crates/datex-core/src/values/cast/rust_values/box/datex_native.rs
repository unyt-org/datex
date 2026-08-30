use core::any::Any;
use crate::preludes::derive::{DatexNative, SharedReferencesCache, Type};
use crate::traits::get_datex_type::GetDatexType;
use crate::types::entity_type::EntityType;
use crate::values::value::value_classification::ValueClassification;

impl<T: DatexNative + GetDatexType> DatexNative
for Box<T>
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn classification(&self, cache: &mut SharedReferencesCache) -> ValueClassification {
        ValueClassification::None
    }
}
