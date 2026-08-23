use crate::shared_values::errors::AccessError;
use crate::traits::value_access::ValueAccess;
use crate::values::core_values::native::NativeCoreValue;
use crate::values::value::ValueContainerOrBorrowedValue;
use crate::values::value_container::value_key::BorrowedValueKey;
use crate::values::value_container::ValueContainer;

impl ValueAccess for NativeCoreValue {
    fn try_get_property(&self, key: BorrowedValueKey) -> Result<ValueContainerOrBorrowedValue<'_>, AccessError> {
        self.value.try_get_property(key)
    }

    fn try_get_property_mut(&mut self, key: BorrowedValueKey) -> Result<&mut ValueContainer, AccessError> {
        self.value.try_get_property_mut(key)
    }
}