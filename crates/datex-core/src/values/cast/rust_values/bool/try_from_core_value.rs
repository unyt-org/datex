use crate::{
    traits::convert_core_value::ConvertCoreValue,
    utils::{goat::Goat, goat_mut::GoatMut},
    values::{
        core_value::CoreValue,
        core_values::boolean::Boolean,
        value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut},
    },
};

impl ConvertCoreValue for bool {
    fn try_from_core_value(value: CoreValue) -> Result<Self, CoreValue> {
        match value {
            CoreValue::Boolean(Boolean(bool)) => Ok(bool),
            CoreValue::Native(native) => {
                native.try_into_value().map_err(CoreValue::Native)
            }
            _ => Err(value),
        }
    }

    fn try_borrow_from_core_value(value: &CoreValue) -> Result<&Self, ()> {
        match value {
            CoreValue::Boolean(Boolean(bool)) => Ok(bool),
            CoreValue::Native(native) => native.try_as().ok_or(()),
            _ => Err(()),
        }
    }

    fn try_borrow_mut_from_core_value(
        value: &mut CoreValue,
    ) -> Result<&mut Self, ()> {
        match value {
            CoreValue::Boolean(Boolean(bool)) => Ok(bool),
            CoreValue::Native(native) => native.try_as_mut().ok_or(()),
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
    use super::*;
    use crate::values::{core_value::CoreValue, core_values::boolean::Boolean};

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
