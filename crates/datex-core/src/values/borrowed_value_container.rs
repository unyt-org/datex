use crate::shared_values::shared_container::SharedContainerValueOrType;
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

/// A variant of the normal [`ValueContainer`] which holds a borrowed reference to a local value instead
/// of an owned value
/// Shared values are still owned.
pub enum BorrowedValueContainer<'a> {
    Local(&'a Value),
    Shared(SharedContainerValueOrType)
}


impl<'a> From<&'a Value> for BorrowedValueContainer<'a> {
    fn from(value: &'a Value) -> Self {
        BorrowedValueContainer::Local(value)
    }
}

impl From<SharedContainerValueOrType> for BorrowedValueContainer<'_> {
    fn from(shared_container: SharedContainerValueOrType) -> Self {
        BorrowedValueContainer::Shared(shared_container)
    }
}

impl<'a> From<&'a ValueContainer> for BorrowedValueContainer<'a> {
    fn from(container: &'a ValueContainer) -> Self {
        match container {
            ValueContainer::Local(value) => BorrowedValueContainer::Local(value),
            ValueContainer::Shared(shared_container) => BorrowedValueContainer::Shared(shared_container.derive_with_max_mutability()),
        }
    }
}