use core::hash::Hash;
use crate::collections::HashMap;
use crate::traits::convert_core_value::ConvertCoreValue;
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::core_value::CoreValue;
use crate::values::core_values::native::DatexNativeBase;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut};

impl<K: DatexNativeBase + Eq + Hash + 'static, V: DatexNativeBase + 'static> ConvertCoreValue for HashMap<K, V> {
    fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue> {
        match value {
            CoreValue::Native(native) => native.try_into_value().map_err(CoreValue::Native),
            _ => Err(value),
        }
    }

    fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()> {
        match value {
            CoreValue::Native(native) => native.try_as().ok_or(()),
            _ => Err(()),
        }
    }

    fn try_borrow_mut_from_core_value(value: &mut CoreValue) -> Result<&mut Self, ()> {
        match value {
            CoreValue::Native(native) => native.try_as_mut().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, K: DatexNativeBase + Eq + Hash + 'static, V: DatexNativeBase + 'static> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, HashMap<K, V>> {
    type Error = ();
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::Native(native) => native.filter_map(|v| v.as_any().downcast_ref::<HashMap<K, V>>()).ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, K: DatexNativeBase + Eq + Hash + 'static, V: DatexNativeBase + 'static> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, HashMap<K, V>> {
    type Error = ();
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::Native(native) => native.filter_map(|v| v.as_any_mut().downcast_mut::<HashMap<K, V>>()).ok_or(()),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::core_value::CoreValue;

    #[test]
    fn try_hash_map_from_core_value() {
        let mut core_value = CoreValue::native(HashMap::<i32, i32>::default());
        let result = core_value.try_as::<HashMap<i32, i32>>();
        assert!(result.is_some());
        
        let result_mut = core_value.try_as_mut::<HashMap<i32, i32>>();
        assert!(result_mut.is_some());
        
        let result_into = core_value.try_into_value::<HashMap<i32, i32>>();
        assert!(result_into.is_ok());
    }
}