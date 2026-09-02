use crate::preludes::derive::SharedReferencesCache;
use crate::traits::classification::Classification;
use crate::traits::static_classification::StaticClassification;
use crate::values::value::Value;
use crate::values::value::value_classification::ValueClassification;

impl Classification for Value {
    fn classification(&self, cache: &mut SharedReferencesCache) -> ValueClassification {
        self.classification.clone()
    }
}