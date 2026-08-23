use std::collections::HashMap;
use std::hash::Hash;
use crate::shared_values::errors::AccessError;
use crate::traits::value_access::ValueAccess;
use crate::values::core_values::native::DatexNative;
use crate::values::value::ValueContainerOrBorrowedValue;
use crate::values::value_container::value_key::BorrowedValueKey;
use crate::values::value_container::ValueContainer;

impl<K, V> ValueAccess for HashMap<K, V>
where
    K: DatexNative + Eq + Hash,
    V: DatexNative,
{
    fn try_get_property(&self, key: BorrowedValueKey) -> Result<ValueContainerOrBorrowedValue<'_>, AccessError> {
        todo!()
    }

    fn try_get_property_mut(&mut self, key: BorrowedValueKey) -> Result<&mut ValueContainer, AccessError> {
        todo!()
    }
}