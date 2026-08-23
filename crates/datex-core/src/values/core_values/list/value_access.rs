use crate::shared_values::errors::{AccessError};
use crate::traits::value_access::ValueAccess;
use crate::values::core_values::list::List;
use crate::values::value::{ValueContainerOrCallable};
use crate::values::value_container::value_key::BorrowedValueKey;
use crate::values::value_container::ValueContainer;

impl ValueAccess for List {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
    ) -> Result<ValueContainerOrCallable<'_>, AccessError> {
        if let Some(index) = key.try_as_index() {
            Ok(ValueContainerOrCallable::ValueContainer(
                self.try_get(index)?,
            ))
        } else {
            Err(AccessError::InvalidIndexKey)
        }
    }

    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
    ) -> Result<&mut ValueContainer, AccessError> {
        if let Some(index) = key.try_as_index() {
            Ok(self.try_get_mut(index)?)
        } else {
            Err(AccessError::InvalidIndexKey)
        }
    }
}