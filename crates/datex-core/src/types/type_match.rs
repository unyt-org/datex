use crate::values::value_container::ValueContainer;

pub trait TypeMatch {
    /// Returns whether self matches another [TypeMatch]
    fn matches(&self, other: &Self) -> bool;

    /// Returns whether a given [ValueContainer] matches self
    fn matched_by_value(&self, value: &ValueContainer) -> bool;
}
