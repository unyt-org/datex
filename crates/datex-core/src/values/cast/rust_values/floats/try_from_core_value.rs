use crate::preludes::derive::{BorrowedCoreValue, BorrowedCoreValueMut};
use crate::utils::goat::Goat;
use crate::utils::goat_mut::GoatMut;
use crate::values::core_value::CoreValue;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

impl TryFrom<CoreValue> for f32 {
    type Error = ();
    fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::TypedDecimal(TypedDecimal::F32(value)) => Ok(value.0),
            CoreValue::Native(native) => native.try_into_value().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, f32> {
    type Error = ();
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::TypedDecimal(value) => {
                value.filter_map(|v| v.borrow_as_f32()).ok_or(())
            }
            BorrowedCoreValue::Native(native) => native.filter_map(|v| v.as_any().downcast_ref::<f32>()).ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, f32> {
    type Error = ();
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::TypedDecimal(value) => {
                value.filter_map(|v| v.borrow_mut_as_f32()).ok_or(())
            }
            BorrowedCoreValueMut::Native(native) => native.filter_map(|v| v.as_any_mut().downcast_mut::<f32>()).ok_or(()),
            _ => Err(()),
        }
    }
}



impl TryFrom<CoreValue> for f64 {
    type Error = ();
    fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
        match value {
            CoreValue::TypedDecimal(TypedDecimal::F64(value)) => Ok(value.0),
            CoreValue::Native(native) => native.try_into_value().ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValue<'a>> for Goat<'a, f64> {
    type Error = ();
    fn try_from(value: BorrowedCoreValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValue::TypedDecimal(value) => {
                value.filter_map(|v| v.borrow_as_f64()).ok_or(())
            }
            BorrowedCoreValue::Native(native) => native.filter_map(|v| v.as_any().downcast_ref::<f64>()).ok_or(()),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<BorrowedCoreValueMut<'a>> for GoatMut<'a, f64> {
    type Error = ();
    fn try_from(value: BorrowedCoreValueMut<'a>) -> Result<Self, Self::Error> {
        match value {
            BorrowedCoreValueMut::TypedDecimal(value) => {
                value.filter_map(|v| v.borrow_mut_as_f64()).ok_or(())
            }
            BorrowedCoreValueMut::Native(native) => native.filter_map(|v| v.as_any_mut().downcast_mut::<f64>()).ok_or(()),
            _ => Err(()),
        }
    }
}
