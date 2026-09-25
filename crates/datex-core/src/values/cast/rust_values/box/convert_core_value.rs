use crate::{
    prelude::*, preludes::derive::DatexNative,
    traits::convert_core_value::ConvertCoreValue,
    values::core_value::CoreValue,
};

impl<T> ConvertCoreValue for Box<T>
where
    Box<T>: DatexNative + 'static,
{
    fn to_core_value(self) -> CoreValue {
        CoreValue::native(self)
    }
    fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue> {
        match value {
            CoreValue::Native(native) => {
                native.try_into_value::<Box<T>>().map_err(CoreValue::Native)
            }
            _ => Err(value),
        }
    }

    fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()> {
        match value {
            CoreValue::Native(native) => native.try_as::<Box<T>>().ok_or(()),
            _ => Err(()),
        }
    }

    fn try_borrow_mut_from_core_value(
        value: &mut CoreValue,
    ) -> Result<&mut Self, ()> {
        match value {
            CoreValue::Native(native) => {
                native.try_as_mut::<Box<T>>().ok_or(())
            }
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_box_from_native_core_value() {
        let core_value = Box::new(42u32).to_core_value();
        let result = core_value.try_into_value::<Box<u32>>().unwrap();
        assert_eq!(*result, 42);
    }
}
