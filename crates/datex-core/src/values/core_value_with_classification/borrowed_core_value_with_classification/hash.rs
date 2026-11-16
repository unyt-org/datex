use core::hash::{Hash, Hasher};
use crate::values::core_value_with_classification::borrowed_core_value_with_classification::BorrowedCoreValueWithClassification;

impl Hash for BorrowedCoreValueWithClassification<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
        self.classification.hash(state);
    }
}
