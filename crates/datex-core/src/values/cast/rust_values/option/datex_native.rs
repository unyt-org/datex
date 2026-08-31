use core::any::Any;
use crate::traits::get_datex_type::GetDatexType;
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::values::core_values::native::DatexNative;
use crate::values::value::value_classification::ValueClassification;

impl<T: DatexNative + GetDatexType> DatexNative
for Option<T>
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn classification(&self, cache: &mut SharedReferencesCache) -> ValueClassification {
        match self {
            Some(value) => value.classification(cache),
            None => ValueClassification::None,
        }
    }
}
