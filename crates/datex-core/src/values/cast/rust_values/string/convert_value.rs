use crate::{
    prelude::*,
    preludes::derive::Text,
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut},
    },
};
use crate::traits::convert_value::ConvertValue;
use crate::values::core_value_with_classification::CoreValueWithClassification;
use crate::values::value::Value;

impl ConvertValue for String {
    fn to_value(self) -> Value {
        CoreValue::Text(Text(self)).into()
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Core(CoreValueWithClassification {inner: CoreValue::Text(Text(string)), ..}) => Ok(string),
            Value::Native(native) => {
                native.try_into_value().map_err(Value::Core)
            }
            _ => Err(value),
        }
    }

    fn try_borrow_from_value(value: &Value) -> Result<&Self, ()> {
        match value {
            Value::Core(CoreValueWithClassification {inner: CoreValue::Text(Text(string)), ..}) => Ok(string),
            Value::Native(native) => native.try_as().ok_or(()),
            _ => Err(()),
        }
    }

    fn try_borrow_mut_from_value(
        value: &mut Value,
    ) -> Result<&mut Self, ()> {
        match value {
            Value::Core(CoreValueWithClassification {inner: CoreValue::Text(Text(string)), ..}) => Ok(string),
            Value::Native(native) => native.try_as_mut().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, String> {
    type Error = ();
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::Text(v) => Ok(v.map(|v| &v.0)),
            BorrowedCoreValue::Native(native) => native
                .filter_map(|v| v.as_any().downcast_ref::<String>())
                .ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, String> {
    type Error = ();
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::Text(v) => Ok(v.map(|v| &mut v.0)),
            BorrowedCoreValueMut::Native(native) => native
                .filter_map(|v| v.as_any_mut().downcast_mut::<String>())
                .ok_or(()),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::core_value::CoreValue;

    #[test]
    fn try_string_from_core_value() {
        let core_value = CoreValue::Text(Text("Hello, World!".to_string()));
        let result = core_value.try_as::<String>();
        assert_eq!(result.unwrap(), "Hello, World!");

        let core_value = CoreValue::Text(Text("Hello, World!".to_string()));
        let result = core_value.try_into_value::<String>();
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn try_borrow_string_from_core_value() {
        let core_value = CoreValue::Text(Text("Hello, World!".to_string()));
        let result = core_value.try_as::<String>();
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn try_borrow_mut_string_from_core_value() {
        let mut core_value = Text("Hello, World!".to_string()).to_value();
        let result = core_value.try_as_mut::<String>();

        let value = result.unwrap();
        value.push_str("!");
        assert_eq!(core_value.try_as::<String>().unwrap(), "Hello, World!!");
    }

    #[test]
    fn try_string_from_native_core_value() {
        let core_value = "Hello, World!".to_string().to_value();
        let result = core_value.try_as::<String>();
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn try_borrow_mut_string_from_native_core_value() {
        let mut core_value = "Hello, World!".to_string().to_value();
        let result = core_value.try_as_mut::<String>();
        result.unwrap().push('!');
        assert_eq!(core_value.try_as::<String>().unwrap(), "Hello, World!!");
    }

    #[test]
    fn try_owned_string_from_native_core_value() {
        let core_value = "Hello, World!".to_string().to_value();
        let result = core_value.try_into_value::<String>();
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn try_string_from_wrong_core_value_fails() {
        let core_value = CoreValue::Null;
        assert!(core_value.try_as::<String>().is_none());
    }

    #[test]
    fn try_borrow_mut_string_from_wrong_core_value_fails() {
        let mut core_value = CoreValue::Null;
        assert!(core_value.try_as_mut::<String>().is_none());
    }
}
