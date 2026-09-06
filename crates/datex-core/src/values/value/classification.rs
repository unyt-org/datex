use crate::{
    preludes::derive::SharedReferencesCache,
    traits::classification::Classification,
    values::value::{Value, value_classification::ValueClassification},
};

impl Classification for Value {
    fn classification(
        &self,
        _cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        self.classification.clone()
    }
}
