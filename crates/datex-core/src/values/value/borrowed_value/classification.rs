use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::traits::classification::Classification;
use crate::values::value::borrowed_value::BorrowedValue;
use crate::values::value::value_classification::unresolved_value_classification::UnresolvedValueClassification;
use crate::values::value::value_classification::ValueClassification;

impl<'a> Classification for BorrowedValue<'a> {
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        match self {
            BorrowedValue::Core(core_value) => core_value.classification(cache),
            BorrowedValue::Native(custom_value) => custom_value.classification(cache),
        }
    }

    fn unresolved_classification(&self) -> UnresolvedValueClassification {
        match self {
            BorrowedValue::Core(core_value) => core_value.unresolved_classification(),
            BorrowedValue::Native(custom_value) => custom_value.unresolved_classification(),
        }
    }
}
