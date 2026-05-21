use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

/// Returns whether self is a superset of a type [T]
pub trait TypeSuperset<T> {
    /// Returns whether self is a superset of type
    fn is_superset_of(&self, other: &T) -> bool;
}

/// Returns whether self is a subset of a type [T]
pub trait TypeSubset<T> {
    /// Returns whether self is a subset of type
    fn is_subset_of(&self, other: &T) -> bool;
}

/// Auto implementation of TypeSubset for any type [T] that implements TypeSuperset for [U]
impl<T, U> TypeSubset<U> for T
where
    U: TypeSuperset<T>,
{
    fn is_subset_of(&self, other: &U) -> bool {
        other.is_superset_of(self)
    }
}


/// Returns whether self satisfies a given [Value] (i.e. value matches self)
pub trait TypeSatisfiesValue {
    /// Returns whether self satisfies a given [Value] (i.e. value matches self)
    fn satisfies_value(&self, value: &Value) -> bool;
}


/// Returns whether self satisfies a given [ValueContainer] (i.e. value_container matches self)
pub trait TypeSatisfiesValueContainer {
        /// Returns whether self satisfies a given [ValueContainer] (i.e. value_container matches self)
    fn satisfies_value_container(&self, value_container: &ValueContainer) -> bool;
}