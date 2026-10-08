use crate::values::value::Value;

pub trait ConvertValue {
    fn to_value(self) -> Value;

    fn try_from_value(value: Value) -> Result<Self, Value>
    where
        Self: Sized;

    fn try_borrow_from_value(value: &Value) -> Result<&Self, ()>
    where
        Self: Sized;

    fn try_borrow_mut_from_value(
        value: &mut Value,
    ) -> Result<&mut Self, ()>
    where
        Self: Sized;
}
