use crate::{
    preludes::derive::{DatexNative, SharedReferencesCache},
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::value::value_classification::ValueClassification,
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

impl<T> StaticClassification for Option<T> where T: DatexNative + 'static {}
