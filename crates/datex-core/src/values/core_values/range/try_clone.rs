use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::range::Range;

impl TryClone for Range {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::Range(self.clone()))
    }
}