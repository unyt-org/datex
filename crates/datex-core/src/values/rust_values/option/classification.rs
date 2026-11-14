use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::classification::Classification,
    values::{
        core_values::native::DatexNative,
        value::value_classification::ValueClassification,
    },
};

impl<T> Classification for Option<T>
where
    T: DatexNative + 'static,
{
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        match self {
            Some(value) => value.classification(cache),
            None => ValueClassification::new_unclassified(),
        }
    }
}
