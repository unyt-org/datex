use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::list::List;

impl TryClone for List {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::List(self.clone()))
    }
}