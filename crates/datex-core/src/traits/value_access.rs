use crate::shared_values::errors::{AccessError};
use crate::values::value::ValueContainerOrBorrowedValue;
use crate::values::value_container::value_key::BorrowedValueKey;
use crate::values::value_container::ValueContainer;
use crate::values::core_values::callable::Callable;

/// Trait for accessing properties of a value as [ValueContainers] or [ValueContainerOrBorrowedValue]. This is used for accessing properties of values in a generic way, such as for maps and structs.
pub trait ValueAccess {
    // FIXME: no ValueContainerOrCallable, generic way to handle borrowed values
    /// Gets a reference to a property on the value if applicable (e.g. for map and structs).
    /// This method returns a [ValueContainerOrBorrowedValue] which can be either a [ValueContainer] or a [Callable] value.
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
    ) -> Result<ValueContainerOrBorrowedValue<'_>, AccessError> {
        Err(AccessError::InvalidOperation("Cannot get property".to_string()))
    }

    /// Gets a mutable reference to a property on the value if applicable (e.g. for map and structs)
    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
    ) -> Result<&mut ValueContainer, AccessError> {
        Err(AccessError::InvalidOperation("Cannot get property".to_string()))
    }
}