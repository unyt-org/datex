use crate::{
    traits::identity::Identity,
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value_container::ValueContainer,
    },
};

/// Identity checks only returns true if two references are identical.
/// Values are never identical to references or other values.
impl<'a> Identity for BorrowedValueContainer<'a> {
    fn identical(&self, other: &Self) -> bool {
        match (self, other) {
            (
                BorrowedValueContainer::Local(_),
                BorrowedValueContainer::Local(_),
            ) => false,
            (
                BorrowedValueContainer::Shared(a),
                BorrowedValueContainer::Shared(b),
            ) => a.identical(b),
            _ => false,
        }
    }
}
