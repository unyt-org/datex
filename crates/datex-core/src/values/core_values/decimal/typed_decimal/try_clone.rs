use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;

impl TryClone for TypedDecimal {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::TypedDecimal(self.clone()))
    }
}