use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::core_value::CoreValue;
use crate::values::core_values::native::DatexNativeBase;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut};

impl<T: DatexNativeBase + 'static> TryFrom<CoreValue> for Vec<T> {
    type Error = ();
    fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Native(native) => native.try_into_value().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, T: DatexNativeBase + 'static> TryFrom<&'a CoreValue> for &'a Vec<T> {
    type Error = ();
    fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Native(native) => native.try_as().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, T: DatexNativeBase + 'static> TryFrom<&'a mut CoreValue> for &'a mut Vec<T> {
    type Error = ();
    fn try_from(value: &'a mut CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Native(native) => native.try_as_mut().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, T: DatexNativeBase + 'static> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, Vec<T>> {
    type Error = ();
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::Native(native) => native.filter_map(|v| v.as_any().downcast_ref::<Vec<T>>()).ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, T: DatexNativeBase + 'static> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, Vec<T>> {
    type Error = ();
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::Native(native) => native.filter_map(|v| v.as_any_mut().downcast_mut::<Vec<T>>()).ok_or(()),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::core_value::CoreValue;
    use crate::values::core_values::boolean::Boolean;

    #[test]
    fn try_vec_from_core_value() {
        let mut core_value = CoreValue::native(Vec::<i32>::default());
        let result = core_value.try_as::<Vec<i32>>();
        assert!(result.is_some());
        
        let result_mut = core_value.try_as_mut::<Vec<i32>>();
        assert!(result_mut.is_some());
        
        let result_into = core_value.try_into_value::<Vec<i32>>();
        assert!(result_into.is_some());
    }
}