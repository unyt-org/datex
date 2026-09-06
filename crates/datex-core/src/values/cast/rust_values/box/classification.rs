use crate::{
    prelude::*,
    preludes::derive::{DatexNative, SharedReferencesCache},
    traits::{
        classification::Classification, get_datex_type::GetDatexType,
        static_classification::StaticClassification,
    },
    values::value::value_classification::ValueClassification,
};

impl<T: DatexNative + GetDatexType> Classification for Box<T> {
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        self.as_ref().classification(cache)
    }
}

impl<T: DatexNative + GetDatexType + StaticClassification> StaticClassification
    for Box<T>
{
    fn has_classification() -> bool {
        T::has_classification()
    }
}
