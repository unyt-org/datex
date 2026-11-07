use crate::{
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    values::{
        value::borrowed_value::{BorrowedValue, BorrowedValueMut},
        value_container::borrowed_value_container::{
            BorrowedValueContainer, BorrowedValueContainerMut,
        },
    },
};
use core::ops::{Deref, DerefMut};

/// Partial equality for ValueContainer is identical to Hash behavior:
/// Identical references are partially equal, value-equal values are also partially equal.
/// A pointer and a value are never partially equal.
impl PartialEq for BorrowedValueContainer<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value_eq(other)
    }
}
impl PartialEq for BorrowedValueContainerMut<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value_eq(other)
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
            ) => {
                &BorrowedValue::from(b.collapsed_value().borrow().deref()) == a
            }
        }
    }
}

impl StructuralEq for BorrowedValueContainerMut<'_> {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                BorrowedValueContainerMut::Local(a),
                BorrowedValueContainerMut::Local(b),
            ) => a.structural_eq(b),
            (
                BorrowedValueContainerMut::Shared(a),
                BorrowedValueContainerMut::Shared(b),
            ) => a.structural_eq(b),
            (
                BorrowedValueContainerMut::Local(a),
                BorrowedValueContainerMut::Shared(b),
            )
            | (
                BorrowedValueContainerMut::Shared(b),
                BorrowedValueContainerMut::Local(a),
            ) => BorrowedValueMut::try_from(
                b.collapsed_value_mut().borrow_mut().deref_mut(),
            )
            .map_or(false, |val| &val == a),
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
            ) => a.value_eq(&BorrowedValue::from(
                b.collapsed_value().borrow().deref(),
            )),
        }
    }
}

impl ValueEq for BorrowedValueContainerMut<'_> {
    fn value_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                BorrowedValueContainerMut::Local(a),
                BorrowedValueContainerMut::Local(b),
            ) => a.value_eq(b),
            (
                BorrowedValueContainerMut::Shared(a),
                BorrowedValueContainerMut::Shared(b),
            ) => a.value_eq(b),
            (
                BorrowedValueContainerMut::Local(a),
                BorrowedValueContainerMut::Shared(b),
            )
            | (
                BorrowedValueContainerMut::Shared(b),
                BorrowedValueContainerMut::Local(a),
            ) => {
                if let Ok(val) = BorrowedValueMut::try_from(
                    b.collapsed_value_mut().borrow_mut().deref_mut(),
                ) {
                    return a.value_eq(&val);
                }
                return false;
            }
        }
    }
}
