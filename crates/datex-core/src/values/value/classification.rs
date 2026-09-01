use crate::preludes::derive::SharedReferencesCache;
use crate::traits::classification::Classification;
use crate::traits::has_classification::HasClassification;
use crate::values::value::Value;
use crate::values::value::value_classification::ValueClassification;

impl Classification for Value {
    fn classification(&self, cache: &mut SharedReferencesCache) -> ValueClassification {
        self.classification.clone()
    }
}