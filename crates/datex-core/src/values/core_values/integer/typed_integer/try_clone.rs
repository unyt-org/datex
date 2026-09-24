use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::integer::typed_integer::TypedInteger;

impl TryClone for TypedInteger {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::TypedInteger(self.clone()))
    }
}