use crate::datex_proxy::DatexProxyType;
use crate::shared_values::errors::{AccessError, IndexOutOfBoundsError};
use crate::traits::value_access::ValueAccess;
use crate::values::core_values::native::{DatexNative, NativeCoreValue};
use crate::values::value::{ValueContainerOrBorrowedValue};
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedValue};
use crate::values::value_container::value_key::BorrowedValueKey;
use crate::values::value_container::ValueContainer;

impl<T> ValueAccess for Vec<T> where T: DatexNative + DatexProxyType {
    fn try_get_property(&self, key: BorrowedValueKey) -> Result<ValueContainerOrBorrowedValue<'_>, AccessError> {
        match key {
            BorrowedValueKey::Index(index) => {
                if let Some(value) = self.get(index as usize) {
                    BorrowedValue {
                        inner: BorrowedCoreValue::Native(NativeCoreValue::from(value)),
                        custom_type: Some(T::datex_type(context).convert_to_definition()),
                    }
                } else {
                    Err(AccessError::IndexOutOfBounds(IndexOutOfBoundsError { index: index as u32 }))
                }
            }
            _ => Err(AccessError::InvalidOperation(format!("Invalid key: {:?}", key))), // TODO: better access errors
        }
    }

    fn try_get_property_mut(&mut self, key: BorrowedValueKey) -> Result<&mut ValueContainer, AccessError> {
        todo!()
    }
}