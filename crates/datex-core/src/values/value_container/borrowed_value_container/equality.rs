use crate::{
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value_container::ValueContainer,
    },
};

/// Partial equality for ValueContainer is identical to Hash behavior:
/// Identical references are partially equal, value-equal values are also partially equal.
/// A pointer and a value are never partially equal.
impl PartialEq for BorrowedValueContainer<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                BorrowedValueContainer::Local(a),
                BorrowedValueContainer::Local(b),
            ) => a == b,
            (
                BorrowedValueContainer::Shared(a),
                BorrowedValueContainer::Shared(b),
            ) => a == b,
            _ => false,
        }
    }
}

/// Structural equality checks the structural equality of the underlying values, collapsing
/// references to their current resolved values.
impl StructuralEq for BorrowedValueContainer<'_> {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                BorrowedValueContainer::Local(a),
                BorrowedValueContainer::Local(b),
            ) => a.structural_eq(b),
            (
                BorrowedValueContainer::Shared(a),
                BorrowedValueContainer::Shared(b),
            ) => a.structural_eq(b),
            (
                BorrowedValueContainer::Local(a),
                BorrowedValueContainer::Shared(b),
            )
            | (
                BorrowedValueContainer::Shared(b),
                BorrowedValueContainer::Local(a),
            ) => &*b.collapsed_value().borrow() == a,
        }
    }
}

/// Value equality checks the value equality of the underlying values, collapsing
/// references to their current resolved values.
impl ValueEq for BorrowedValueContainer<'_> {
    fn value_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                BorrowedValueContainer::Local(a),
                BorrowedValueContainer::Local(b),
            ) => a.value_eq(b),
            (
                BorrowedValueContainer::Shared(a),
                BorrowedValueContainer::Shared(b),
            ) => a.value_eq(b),
            (
                BorrowedValueContainer::Local(a),
                BorrowedValueContainer::Shared(b),
            )
            | (
                BorrowedValueContainer::Shared(b),
                BorrowedValueContainer::Local(a),
            ) => a.value_eq(&*b.collapsed_value().borrow()),
        }
    }
}
