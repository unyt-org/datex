use crate::preludes::derive::Text;
use crate::traits::convert_core_value::ConvertCoreValue;
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::core_value::CoreValue;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut};
use crate::prelude::*;

impl ConvertCoreValue for String {
    fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue> {
        match value {
            CoreValue::Text(Text(string)) => Ok(string),
            CoreValue::Native(native) => native.try_into_value().map_err(CoreValue::Native),
            _ => Err(value),
        }
    }

    fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()> {
        match value {
            CoreValue::Text(Text(string)) => Ok(string),
            CoreValue::Native(native) => native.try_as().ok_or(()),
            _ => Err(()),
        }
    }

    fn try_borrow_mut_from_core_value(value: &mut CoreValue) -> Result<&mut Self, ()>
    {
        match value {
            CoreValue::Text(Text(string)) => Ok(string),
            CoreValue::Native(native) => native.try_as_mut().ok_or(()),
            _ => Err(()),
        }
    }
}


impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, String> {
    type Error = ();
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::Text(v) => Ok(v.map(|v| &v.0)),
            BorrowedCoreValue::Native(native) => native.filter_map(|v| v.as_any().downcast_ref::<String>()).ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, String> {
    type Error = ();
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::Text(v) => Ok(v.map(|v| &mut v.0)),
            BorrowedCoreValueMut::Native(native) => native.filter_map(|v| v.as_any_mut().downcast_mut::<String>()).ok_or(()),
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
    fn try_string_from_core_value() {
        let mut core_value = CoreValue::Text(Text("Hello, World!".to_string()));
        let result = core_value.try_as::<String>();
        assert_eq!(*result.unwrap(), "Hello, World!");
        
        let result_mut = core_value.try_as_mut::<String>();
        assert_eq!(*result_mut.unwrap(), "Hello, World!");
        
        let result_into = core_value.try_into_value::<String>();
        assert_eq!(result_into.unwrap(), "Hello, World!");
    }
}