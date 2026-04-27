use crate::values::{
    core_value::CoreValue, value_container::error::ValueError,
};
use core::{ops::Neg, result::Result};

impl Neg for CoreValue {
    type Output = Result<CoreValue, ValueError>;

    fn neg(self) -> Self::Output {
        match self {
            CoreValue::TypedInteger(int) => {
                Ok(CoreValue::TypedInteger(int.neg()?))
            }
            CoreValue::Integer(int) => Ok(CoreValue::Integer(int.neg())),
            CoreValue::TypedDecimal(decimal) => {
                Ok(CoreValue::TypedDecimal(decimal.neg()))
            }
            CoreValue::Decimal(decimal) => {
                Ok(CoreValue::Decimal(decimal.neg()))
            }
            _ => Err(ValueError::InvalidOperation), // Negation not applicable for other types
        }
    }
}
