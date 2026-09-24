use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::decimal::Decimal;

impl TryClone for Decimal {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::Decimal(self.clone()))
    }
}