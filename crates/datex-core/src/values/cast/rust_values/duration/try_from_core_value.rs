use core::time::Duration;
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::core_value::CoreValue;
use crate::values::core_values::boolean::Boolean;
use crate::values::value::borrowed_value::{BorrowedCoreValue, BorrowedCoreValueMut};

impl TryFrom<CoreValue> for Duration {
    type Error = ();
    fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Native(native) => native.try_into_value().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<&'a CoreValue> for &'a Duration {
    type Error = ();
    fn try_from(value: &'a CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Native(native) => native.try_as().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<&'a mut CoreValue> for &'a mut Duration {
    type Error = ();
    fn try_from(value: &'a mut CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::Native(native) => native.try_as_mut().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, Duration> {
    type Error = ();
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::Native(native) => native.filter_map(|v| v.as_any().downcast_ref::<Duration>()).ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, Duration> {
    type Error = ();
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::Native(native) => native.filter_map(|v| v.as_any_mut().downcast_mut::<Duration>()).ok_or(()),
            _ => Err(()),
        }
    }
}