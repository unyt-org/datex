use crate::{
    preludes::derive::SharedReferencesCache,
    traits::classification::Classification,
    values::value::{Value, value_classification::ValueClassification},
};
use crate::preludes::derive::CoreValue;
use crate::values::core_value_with_classification::CoreValueWithClassification;
use crate::values::value::value_classification::unresolved_value_classification::UnresolvedValueClassification;

impl Classification for CoreValueWithClassification {
    fn classification(
        &self,
        _cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        self.classification.clone()
    }

    fn unresolved_classification(&self) -> UnresolvedValueClassification {
        self.classification.clone().into()
    }
}
