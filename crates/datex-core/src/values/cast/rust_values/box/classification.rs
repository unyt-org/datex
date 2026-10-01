use crate::{
    prelude::*,
    preludes::derive::{DatexNative, SharedReferencesCache},
    traits::{
        classification::Classification, get_datex_type::GetDatexType,
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
