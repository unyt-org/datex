use crate::{
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
};
use crate::values::core_value_with_classification::borrowed_core_value_with_classification::BorrowedCoreValueWithClassification;

impl PartialEq for BorrowedCoreValueWithClassification<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.structural_eq(&other.inner)
            && self.classification == other.classification
    }
}

impl StructuralEq for BorrowedCoreValueWithClassification<'_> {
    fn structural_eq(&self, other: &Self) -> bool {
        self.inner.structural_eq(&other.inner)
            && self.classification.structural_eq(&other.classification)
    }
}

impl ValueEq for BorrowedCoreValueWithClassification<'_> {
    fn value_eq(&self, other: &Self) -> bool {
        self.inner.value_eq(&other.inner)
            && self.classification.value_eq(&other.classification)
    }
}
