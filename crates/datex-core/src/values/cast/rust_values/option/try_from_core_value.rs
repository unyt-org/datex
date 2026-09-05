use crate::traits::convert_core_value::ConvertCoreValue;
use crate::values::core_value::CoreValue;
use crate::values::core_values::native::DatexNativeBase;

impl<T> ConvertCoreValue for Option<T>
where
    T: DatexNativeBase + 'static,
{
    fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue> {
        match value {
            CoreValue::Null => Ok(None),
            CoreValue::Native(native) => native.try_into_value::<T>().map(Some).map_err(CoreValue::Native),
            _ => Err(value),
        }
    }

    fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()> {
        Err(())
    }

    fn try_borrow_mut_from_core_value(value: &mut CoreValue) -> Result<&mut Self, ()> {
        Err(())
    }
}

impl<'a, T> TryFrom<&'a CoreValue> for Option<&'a T>
where
    T: DatexNativeBase + 'static,
{
    type Error = ();
    fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Null => Ok(None),
            CoreValue::Native(native) => native.try_as::<T>().ok_or(()).map(Some),
            _ => Err(()),
        }
    }
}

impl<'a, T> TryFrom<&'a mut CoreValue> for Option<&'a mut T>
where
    T: DatexNativeBase + 'static,
{
    type Error = ();
    fn try_from(value: &'a mut CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Null => Ok(None),
            CoreValue::Native(native) => native.try_as_mut::<T>().ok_or(()).map(Some),
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
    fn try_bool_from_core_value() {
        let mut core_value = CoreValue::Boolean(Boolean(true));
        let result = core_value.try_as::<bool>();
        assert_eq!(*result.unwrap(), true);
        
        let result_mut = core_value.try_as_mut::<bool>();
        assert_eq!(*result_mut.unwrap(), true);
        
        let result_into = core_value.try_into_value::<bool>();
        assert_eq!(result_into.unwrap(), true);
    }
}