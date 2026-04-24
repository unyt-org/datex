use crate::{
    traits::{
        identity::Identity, structural_eq::StructuralEq, value_eq::ValueEq,
    },
    values::value_container::ValueContainer,
};

/// Identity checks only returns true if two references are identical.
/// Values are never identical to references or other values.
impl Identity for ValueContainer {
    fn identical(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueContainer::Local(_), ValueContainer::Local(_)) => false,
            (ValueContainer::Shared(a), ValueContainer::Shared(b)) => {
                a.identical(b)
            }
            _ => false,
        }
    }
}
