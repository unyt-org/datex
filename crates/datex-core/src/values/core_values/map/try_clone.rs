use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::map::Map;

impl TryClone for Map {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::Map(self.clone()))
    }
}