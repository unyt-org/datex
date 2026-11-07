use crate::{
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    values::core_value::borrowed_core_value::{
        BorrowedCoreValue, BorrowedCoreValueMut,
    },
};

impl StructuralEq for BorrowedCoreValue<'_> {
    fn structural_eq(&self, other: &Self) -> bool {
        // TODO: optimize?
        self.clone_to_core_value()
            .structural_eq(&other.clone_to_core_value())
    }
}

impl ValueEq for BorrowedCoreValue<'_> {
    fn value_eq(&self, other: &Self) -> bool {
        self.clone_to_core_value()
            .structural_eq(&other.clone_to_core_value())
    }
}

impl PartialEq for BorrowedCoreValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.clone_to_core_value()
            .structural_eq(&other.clone_to_core_value())
    }
}

impl Eq for BorrowedCoreValue<'_> {}

impl StructuralEq for BorrowedCoreValueMut<'_> {
    fn structural_eq(&self, other: &Self) -> bool {
        // TODO: optimize?
        if let Some(lhs) = self.try_clone_to_core_value()
            && let Some(rhs) = other.try_clone_to_core_value()
        {
            lhs.structural_eq(&rhs)
        } else {
            false
        }
    }
}

impl ValueEq for BorrowedCoreValueMut<'_> {
    fn value_eq(&self, other: &Self) -> bool {
        if let Some(lhs) = self.try_clone_to_core_value()
            && let Some(rhs) = other.try_clone_to_core_value()
        {
            lhs.structural_eq(&rhs)
        } else {
            false
        }
    }
}

impl PartialEq for BorrowedCoreValueMut<'_> {
    fn eq(&self, other: &Self) -> bool {
        if let Some(lhs) = self.try_clone_to_core_value()
            && let Some(rhs) = other.try_clone_to_core_value()
        {
            lhs.structural_eq(&rhs)
        } else {
            false
        }
    }
}

impl Eq for BorrowedCoreValueMut<'_> {}
