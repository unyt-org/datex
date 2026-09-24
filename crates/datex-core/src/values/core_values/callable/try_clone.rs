use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::callable::Callable;

impl TryClone for Callable {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::Callable(self.clone()))
    }
}