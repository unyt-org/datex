use crate::shared_values::errors::{AccessError, IndexOutOfBoundsError};
use crate::traits::value_access::ValueAccess;
use crate::values::core_values::native::DatexNative;
use crate::values::value::{Value, ValueContainerOrCallable};
use crate::values::value_container::value_key::BorrowedValueKey;
use crate::values::value_container::ValueContainer;

impl<T> ValueAccess for Vec<T> where T: DatexNative {
    fn try_get_property(&self, key: BorrowedValueKey) -> Result<ValueContainerOrCallable<'_>, AccessError> {
        match key {
            BorrowedValueKey::Index(index) => {
                if let Some(value) = self.get(index as usize) {
                    Ok(ValueContainer::Local(Value::native(value)).into())
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