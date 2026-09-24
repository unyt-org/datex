use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::text::Text;

impl TryClone for Text {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::Text(self.clone()))
    }
}