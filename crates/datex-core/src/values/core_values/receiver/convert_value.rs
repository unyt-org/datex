use crate::{
    traits::convert_value::ConvertValue,
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_values::native::DatexNativeBase,
        value::{
            Value,
            borrowed_value::{BorrowedValue, BorrowedValueMut},
        },
    },
};
use crate::values::core_values::receiver::Receiver;
impl<T: DatexNativeBase + 'static> ConvertValue for Receiver<T> {
    fn to_value(self) -> Value {
        Value::native(self)
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

    fn try_borrow_mut_from_value(value: &mut Value) -> Result<&mut Self, ()> {
        match value {
            Value::Native(native) => native.try_as_mut().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, T: DatexNativeBase + 'static> TryFrom<BorrowedValue<'a>>
    for Goat<'a, Receiver<T>>
{
    type Error = ();
    fn try_from(value: BorrowedValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedValue::Native(native) => native
                .filter_map(|v| v.as_any().downcast_ref::<Receiver<T>>())
                .ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a, T: DatexNativeBase + 'static> TryFrom<BorrowedValueMut<'a>>
    for GoatMut<'a, Receiver<T>>
{
    type Error = ();
    fn try_from(value: BorrowedValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedValueMut::Native(native) => native
                .filter_map(|v| v.as_any_mut().downcast_mut::<Receiver<T>>())
                .ok_or(()),
            _ => Err(()),
        }
    }
}