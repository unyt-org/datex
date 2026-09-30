use super::CoreValue;
use crate::{
    preludes::derive::SharedReferencesCache,
    traits::classification::Classification,
    values::value::value_classification::ValueClassification,
};
impl Classification for CoreValue {
    /// Native core values might have their own [ValueClassification], for these cases the value is returned.
    /// For all other variants, [ValueClassification::new_unclassified()] is returned.
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        match &self {
            CoreValue::Native(native) => native.classification(cache),
            _ => ValueClassification::new_unclassified(),
        }
    }
}
