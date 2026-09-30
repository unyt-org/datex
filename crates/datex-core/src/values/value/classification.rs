use crate::{
    preludes::derive::SharedReferencesCache,
    traits::classification::Classification,
    values::value::{Value, value_classification::ValueClassification},
};

impl Classification for Value {
    /// Give back the classification of the value. If the classification is `None`,
    /// return the classification of the inner value (if it is a native value, it might have its own classification)
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        match &self.classification {
            ValueClassification::None => self.inner.classification(cache),
            other => other.clone(),
        }
    }
}
