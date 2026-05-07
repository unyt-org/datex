use crate::{
    traits::{structural_eq::StructuralEq, value_eq::ValueEq},
    values::value_container::ValueContainer,
};

/// Partial equality for ValueContainer is identical to Hash behavior:
/// Identical references are partially equal, value-equal values are also partially equal.
/// A pointer and a value are never partially equal.
impl PartialEq for ValueContainer {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueContainer::Local(a), ValueContainer::Local(b)) => a == b,
            (ValueContainer::Shared(a), ValueContainer::Shared(b)) => a == b,
            _ => false,
        }
    }
}

/// Structural equality checks the structural equality of the underlying values, collapsing
/// references to their current resolved values.
impl StructuralEq for ValueContainer {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueContainer::Local(a), ValueContainer::Local(b)) => {
                a.structural_eq(b)
            }
            (ValueContainer::Shared(a), ValueContainer::Shared(b)) => {
                a.structural_eq(b)
            }
            (ValueContainer::Local(a), ValueContainer::Shared(b))
            | (ValueContainer::Shared(b), ValueContainer::Local(a)) => {
                b.with_collapsed_value_mut(|b| a.structural_eq(b))
            }
        }
    }
}

/// Value equality checks the value equality of the underlying values, collapsing
/// references to their current resolved values.
impl ValueEq for ValueContainer {
    fn value_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueContainer::Local(a), ValueContainer::Local(b)) => {
                a.value_eq(b)
            }
            (ValueContainer::Shared(a), ValueContainer::Shared(b)) => {
                a.value_eq(b)
            }
            (ValueContainer::Local(a), ValueContainer::Shared(b))
            | (ValueContainer::Shared(b), ValueContainer::Local(a)) => {
                b.with_collapsed_value_mut(|b| a.value_eq(b))
            }
        }
    }
}
