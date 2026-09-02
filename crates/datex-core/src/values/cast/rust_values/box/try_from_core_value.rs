use crate::traits::convert_core_value::ConvertCoreValue;
use crate::values::core_value::CoreValue;
use crate::values::core_values::native::DatexNativeBase;


impl<T> ConvertCoreValue for Box<T>
where
    T: DatexNativeBase + 'static,
{
    fn try_from_core_value(value: CoreValue) -> Result<Self, ()> {
        match value {
            CoreValue::Native(native) => native.try_into_value::<T>().ok_or(()).map(Box::new),
            _ => Err(()),
        }
    }
}