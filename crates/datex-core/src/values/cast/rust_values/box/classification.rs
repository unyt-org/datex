use crate::preludes::derive::{DatexNative, SharedReferencesCache};
use crate::traits::classification::Classification;
use crate::traits::get_datex_type::GetDatexType;
use crate::traits::has_classification::HasClassification;
use crate::values::value::value_classification::ValueClassification;

impl<T: DatexNative + GetDatexType> Classification
for Box<T>
{
    fn classification(&self, cache: &mut SharedReferencesCache) -> ValueClassification {
        self.as_ref().classification(cache)
    }
}

impl<T: DatexNative + GetDatexType + HasClassification> HasClassification for Box<T> {
    fn has_classification() -> bool {
        T::has_classification()
    }
}