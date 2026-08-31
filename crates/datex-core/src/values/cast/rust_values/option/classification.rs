use crate::preludes::derive::SharedReferencesCache;
use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::core_values::native::DatexNativeBase;
use crate::values::value::value_classification::ValueClassification;

impl<T> Classification for Option<T>
where
    T: DatexNativeBase + 'static,
{
    fn classification(&self, cache: &mut SharedReferencesCache) -> ValueClassification {
        match self {
            Some(value) => value.classification(cache),
            None => ValueClassification::None,
        }
    }
}


impl<T> HasClassification for Option<T>
where
    T: DatexNativeBase + 'static,
{}
