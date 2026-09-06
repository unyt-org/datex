use crate::{
    preludes::derive::SharedReferencesCache,
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
    values::{
        core_values::native::DatexNativeBase,
        value::value_classification::ValueClassification,
    },
};

impl<T> Classification for Option<T>
where
    T: DatexNativeBase + 'static,
{
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        match self {
            Some(value) => value.classification(cache),
            None => ValueClassification::None,
        }
    }
}

impl<T> StaticClassification for Option<T> where T: DatexNativeBase + 'static {}
