use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

impl TryClone for f32 {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::TypedDecimal(TypedDecimal::F32(self.clone().into())))
    }
}

impl TryClone for f64 {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::TypedDecimal(TypedDecimal::F64(self.clone().into())))
    }
}