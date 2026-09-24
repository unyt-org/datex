use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::integer::Integer;

impl TryClone for Integer {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::Integer(self.clone()))
    }
}