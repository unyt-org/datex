use crate::{
    preludes::derive::DatexNative,
    traits::convert_core_value::ConvertCoreValue,
    values::{core_value::CoreValue, core_values::native::DatexNativeBase},
};

impl<T> ConvertCoreValue for Option<T>
where
    T: DatexNative + ConvertCoreValue + 'static,
{
    fn to_core_value(self) -> CoreValue {
        match self {
            Some(value) => CoreValue::native(value),
            None => CoreValue::Null,
        }
    }
    fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue> {
        match value {
            CoreValue::Null => Ok(None),
            value => value.try_into_value::<T>().map(Some),
        }
    }

    fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()> {
        // We could return CoreValue::Null => Ok(&None) here, but to make the API consistent with the mutable version and avoid returning a reference to a temporary Option<T>, we simply return an error.
        Err(())
    }

    fn try_borrow_mut_from_core_value(
        value: &mut CoreValue,
    ) -> Result<&mut Self, ()> {
        // We can not cover the CoreValue::Null here because we need to return a mutable reference to an Option<T>
        // We can not call value.try_as_mut::<Option<T>>().ok_or(()) and basicially forced, to do nothing here.
        Err(())
    }
}

impl<'a, T> TryFrom<&'a CoreValue> for Option<&'a T>
where
    T: DatexNativeBase + ConvertCoreValue + 'static,
{
    type Error = ();
    fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Null => Ok(None),
            _ => value.try_as::<T>().map(Some).ok_or(()),
        }
    }
}

impl<'a, T> TryFrom<&'a mut CoreValue> for Option<&'a mut T>
where
    T: DatexNativeBase + ConvertCoreValue + 'static,
{
    type Error = ();
    fn try_from(value: &'a mut CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Null => Ok(None),
            _ => value.try_as_mut::<T>().map(Some).ok_or(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        traits::convert_core_value::ConvertCoreValue,
        values::{
            core_value::CoreValue,
            core_values::integer::typed_integer::TypedInteger,
        },
    };

    #[test]
    fn try_option_from_null() {
        let core_value = CoreValue::Null;
        let result = Option::<u32>::try_from_core_value(core_value);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn try_option_from_native() {
        let core_value = 42u32.to_core_value();
        let result = Option::<u32>::try_from_core_value(core_value);
        assert_eq!(result.unwrap(), Some(42));
    }

    #[test]
    fn try_option_from_wrong_core_value() {
        let core_value = CoreValue::TypedInteger(TypedInteger::I32(42));
        let result = Option::<u32>::try_from_core_value(core_value);
        assert!(result.is_err());
    }

    #[test]
    fn try_option_ref_from_null() {
        let core_value = CoreValue::Null;
        let result = Option::<&u32>::try_from(&core_value);
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn try_option_ref_from_native() {
        let core_value = 42u32.to_core_value();
        let result = Option::<&u32>::try_from(&core_value);
        assert_eq!(*result.unwrap().unwrap(), 42);
    }

    #[test]
    fn try_option_ref_from_wrong_core_value() {
        let core_value = CoreValue::TypedInteger(TypedInteger::I32(42));
        let result = Option::<&u32>::try_from(&core_value);
        assert!(result.is_err());
    }

    #[test]
    fn try_option_mut_ref_from_null() {
        let mut core_value = CoreValue::Null;
        let result = Option::<&mut u32>::try_from(&mut core_value);
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn try_option_mut_ref_from_native() {
        let mut core_value = 42u32.to_core_value();
        let result = Option::<&mut u32>::try_from(&mut core_value);
        let value = result.unwrap().unwrap();
        *value = 100;
        assert_eq!(*core_value.try_as::<u32>().unwrap(), 100);
    }

    #[test]
    fn try_option_mut_ref_from_wrong_core_value() {
        let mut core_value = CoreValue::TypedInteger(TypedInteger::I32(42));
        let result = Option::<&mut u32>::try_from(&mut core_value);
        assert!(result.is_err());
    }
}
