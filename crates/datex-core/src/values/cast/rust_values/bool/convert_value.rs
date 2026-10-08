use crate::{
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::boolean::Boolean,
        value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut},
    },
};
use crate::preludes::derive::Value;
use crate::traits::convert_value::ConvertValue;
use crate::values::core_value_with_classification::CoreValueWithClassification;

impl ConvertValue for bool {
    fn to_value(self) -> Value {
        CoreValue::Boolean(Boolean(self)).into()
    }
    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Core(CoreValueWithClassification {inner: CoreValue::Boolean(Boolean(bool)), ..}) => Ok(bool),
            Value::Native(native) => {
                native.try_into_value().map_err(Value::Native)
            }
            _ => Err(value),
        }
    }

    fn try_borrow_from_value(value: &Value) -> Result<&Self, ()> {
        match value {
            Value::Core(CoreValueWithClassification {inner: CoreValue::Boolean(Boolean(bool)), ..}) => Ok(bool),
            Value::Native(native) => native.try_as().ok_or(()),
            _ => Err(()),
        }
    }

    fn try_borrow_mut_from_value(
        value: &mut Value,
    ) -> Result<&mut Self, ()> {
        match value {
            Value::Core(CoreValueWithClassification {inner: CoreValue::Boolean(Boolean(bool)), ..}) => Ok(bool),
            Value::Native(native) => native.try_as_mut().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, bool> {
    type Error = ();
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::Boolean(v) => Ok(v.map(|v| &v.0)),
            BorrowedCoreValue::Native(native) => native
                .filter_map(|v| v.as_any().downcast_ref::<bool>())
                .ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, bool> {
    type Error = ();
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::Boolean(v) => Ok(v.map(|v| &mut v.0)),
            BorrowedCoreValueMut::Native(native) => native
                .filter_map(|v| v.as_any_mut().downcast_mut::<bool>())
                .ok_or(()),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        preludes::derive::{BorrowedCoreValue, BorrowedCoreValueMut},
        utils::{goat::Goat, goat_mut::GoatMut},
        values::{core_value::CoreValue, core_values::boolean::Boolean},
    };
    use crate::traits::convert_value::ConvertValue;

    #[test]
    fn try_bool_from_core_value() {
        let core_value = CoreValue::Boolean(Boolean(true)).to_value();
        let result = core_value.try_as::<bool>();
        assert!(*result.unwrap());

        let core_value = CoreValue::Boolean(Boolean(false)).to_value();
        let result = core_value.try_into_value::<bool>();
        assert!(!result.unwrap());
    }

    #[test]
    fn try_borrow_bool_from_core_value() {
        let core_value = CoreValue::Boolean(Boolean(true)).to_value();
        let result = core_value.try_as::<bool>();
        assert!(*result.unwrap());
    }

    #[test]
    fn try_borrow_mut_bool_from_core_value() {
        let mut core_value = CoreValue::Boolean(Boolean(false)).to_value();
        let result = core_value.try_as_mut::<bool>();
        *result.unwrap() = true;
        assert_eq!(core_value, CoreValue::Boolean(Boolean(true)).to_value());
    }

    #[test]
    fn try_bool_from_native_core_value() {
        let core_value = true.to_value();
        let result = core_value.try_as::<bool>();
        assert!(*result.unwrap());
    }

    #[test]
    fn try_borrow_mut_bool_from_native_core_value() {
        let mut core_value = false.to_value();
        let result = core_value.try_as_mut::<bool>();
        *result.unwrap() = true;
        assert!(*core_value.try_as::<bool>().unwrap());
    }

    #[test]
    fn try_owned_bool_from_native_core_value() {
        let core_value = true.to_value();
        let result = core_value.try_into_value::<bool>();
        assert!(result.unwrap());
    }

    #[test]
    fn try_bool_from_wrong_core_value_fails() {
        let core_value = CoreValue::Null.to_value();
        assert!(core_value.try_as::<bool>().is_none());
    }

    #[test]
    fn try_borrow_mut_bool_from_wrong_core_value_fails() {
        let mut core_value = CoreValue::Null.to_value();
        assert!(core_value.try_as_mut::<bool>().is_none());
    }

    #[test]
    fn try_owned_bool_from_wrong_core_value_fails() {
        let core_value = CoreValue::Null.to_value();
        assert!(core_value.try_into_value::<bool>().is_err());
    }

    #[test]
    fn try_borrowed_core_value_bool() {
        let core_value = CoreValue::Boolean(Boolean(true));

        let borrowed = BorrowedCoreValue::from(&core_value);
        let result = Goat::<bool>::try_from(borrowed);
        assert!(*result.unwrap());
    }

    #[test]
    fn try_borrowed_core_value_mut_bool() {
        let mut core_value = CoreValue::Boolean(Boolean(false));
        let borrowed = BorrowedCoreValueMut::from(&mut core_value);
        let mut result = GoatMut::<bool>::try_from(borrowed).unwrap();
        *result = true;
        drop(result);
        assert_eq!(core_value, CoreValue::Boolean(Boolean(true)));
    }

    #[test]
    fn try_borrowed_core_value_wrong_type_fails() {
        let core_value = CoreValue::Null;
        let borrowed = BorrowedCoreValue::from(&core_value);
        assert!(Goat::<bool>::try_from(borrowed).is_err());
    }

    #[test]
    fn try_borrowed_core_value_mut_wrong_type_fails() {
        let mut core_value = CoreValue::Null;
        let borrowed = BorrowedCoreValueMut::from(&mut core_value);
        assert!(GoatMut::<bool>::try_from(borrowed).is_err());
    }
}
