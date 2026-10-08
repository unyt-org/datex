use crate::{
    traits::convert_value::ConvertValue,
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
    },
};
use core::time::Duration;
use crate::values::value::borrowed_value::{BorrowedValue, BorrowedValueMut};
use crate::values::value::Value;

impl ConvertValue for Duration {
    fn to_value(self) -> Value {
        todo!()
    }
    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Native(native) => {
                native.try_into_value().map_err(Value::Native)
            }
            _ => Err(value),
        }
    }

    fn try_borrow_from_value(value: &Value) -> Result<&Self, ()> {
        match value {
            Value::Native(native) => native.try_as().ok_or(()),
            _ => Err(()),
        }
    }

    fn try_borrow_mut_from_value(
        value: &mut Value,
    ) -> Result<&mut Self, ()> {
        match value {
            Value::Native(native) => native.try_as_mut().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedValue<'a>> for Goat<'a, Duration> {
    type Error = ();
    fn try_from(value: BorrowedValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedValue::Native(native) => native
                .filter_map(|v| v.as_any().downcast_ref::<Duration>())
                .ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedValueMut<'a>> for GoatMut<'a, Duration> {
    type Error = ();
    fn try_from(value: BorrowedValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedValueMut::Native(native) => native
                .filter_map(|v| v.as_any_mut().downcast_mut::<Duration>())
                .ok_or(()),
            _ => Err(()),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        traits::convert_value::ConvertValue,
        utils::{goat::Goat, goat_mut::GoatMut},
        values::{
            core_value::CoreValue,
        },
    };
    use core::time::Duration;
    use crate::values::value::borrowed_value::{BorrowedValue, BorrowedValueMut};

    #[test]
    fn try_duration_from_native_core_value() {
        let duration = Duration::from_secs(10);
        let core_value = duration.to_value();
        assert_eq!(core_value.try_into_value::<Duration>().unwrap(), duration);
    }

    #[test]
    fn try_borrow_duration_from_native_core_value() {
        let duration = Duration::from_secs(10);
        let core_value = duration.to_value();
        assert_eq!(*core_value.try_as::<Duration>().unwrap(), duration);
    }

    #[test]
    fn try_borrow_mut_duration_from_native_core_value() {
        let mut core_value = Duration::from_secs(10).to_value();
        *core_value.try_as_mut::<Duration>().unwrap() = Duration::from_secs(20);
        assert_eq!(
            *core_value.try_as::<Duration>().unwrap(),
            Duration::from_secs(20)
        );
    }

    #[test]
    fn try_duration_from_wrong_core_value_fails() {
        let core_value = CoreValue::Null.to_value();
        assert!(core_value.try_as::<Duration>().is_none());
        assert!(core_value.try_into_value::<Duration>().is_err());
    }

    #[test]
    fn try_borrowed_core_value_duration() {
        let duration = Duration::from_secs(10);
        let core_value = duration.to_value();
        let borrowed = BorrowedValue::from(&core_value);
        let result = Goat::<Duration>::try_from(borrowed).unwrap();
        assert_eq!(*result, duration);
    }

    #[test]
    fn try_borrowed_core_value_mut_duration() {
        let mut core_value = Duration::from_secs(10).to_value();
        let borrowed = BorrowedValueMut::from(&mut core_value);
        let mut result = GoatMut::<Duration>::try_from(borrowed).unwrap();
        *result = Duration::from_secs(20);
        drop(result);
        assert_eq!(
            *core_value.try_as::<Duration>().unwrap(),
            Duration::from_secs(20)
        );
    }
}
