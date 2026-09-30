use crate::{
    preludes::derive::SharedReferencesCache,
    traits::classification::Classification,
    values::value::{Value, value_classification::ValueClassification},
};

impl Classification for Value {
    /// Give back the classification of the value.
    /// Merges the classification of the inner value with the classification of the [Value].
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        self.classification
            .merge(self.inner.classification(cache))
    }
}
