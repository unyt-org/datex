use crate::shared_values::errors::{AccessError};
use crate::traits::value_access::ValueAccess;
use crate::values::core_values::map::Map;
use crate::values::value::{ValueContainerOrCallable};
use crate::values::value_container::value_key::BorrowedValueKey;
use crate::values::value_container::ValueContainer;

impl ValueAccess for Map {
    fn try_get_property(
        &self,
        key: BorrowedValueKey,
    ) -> Result<ValueContainerOrCallable<'_>, AccessError> {
        Ok(ValueContainerOrCallable::ValueContainer(self.try_get(key)?))
    }

    fn try_get_property_mut(
        &mut self,
        key: BorrowedValueKey,
    ) -> Result<&mut ValueContainer, AccessError> {
        Ok(self.try_get_mut(key)?)
    }
}