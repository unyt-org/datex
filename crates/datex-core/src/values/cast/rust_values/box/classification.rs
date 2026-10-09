use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{classification::Classification, get_datex_type::GetDatexType},
    values::{
        core_values::native::DatexNative,
        value::value_classification::ValueClassification,
    },
};

impl<T: DatexNative + GetDatexType> Classification for Box<T> {
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        self.as_ref().classification(cache)
    }
}
