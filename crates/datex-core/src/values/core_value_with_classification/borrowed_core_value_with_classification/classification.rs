use crate::runtime::cache::shared_references_cache::SharedReferencesCache;
use crate::traits::classification::Classification;
use crate::values::core_value_with_classification::borrowed_core_value_with_classification::BorrowedCoreValueWithClassification;
use crate::values::value::value_classification::unresolved_value_classification::UnresolvedValueClassification;
use crate::values::value::value_classification::ValueClassification;

impl<'a> Classification for BorrowedCoreValueWithClassification<'a> {
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
