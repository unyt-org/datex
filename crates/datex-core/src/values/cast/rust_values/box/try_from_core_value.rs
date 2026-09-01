use crate::values::core_value::CoreValue;
use crate::values::core_values::native::DatexNativeBase;

impl<T> TryFrom<CoreValue> for Box<T>
where
    T: DatexNativeBase + 'static,
{
    type Error = ();
    fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Native(native) => native.try_into_value::<T>().ok_or(()).map(Box::new),
            _ => Err(()),
        }
    }
}