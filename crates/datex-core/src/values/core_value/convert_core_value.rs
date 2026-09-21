use crate::traits::convert_core_value::ConvertCoreValue;
use crate::values::core_value::CoreValue;

impl ConvertCoreValue for CoreValue {
    fn to_core_value(self) -> CoreValue {
        self
    }

    fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue>
    where
        Self: Sized
    {
        Ok(value)
    }

    fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()> {
        Ok(value)
    }

    fn try_borrow_mut_from_core_value(value: &mut CoreValue) -> Result<&mut Self, ()> {
        Ok(value)
    }
}