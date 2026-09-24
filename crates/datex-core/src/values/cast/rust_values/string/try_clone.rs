use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::prelude::*;

impl TryClone for String {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::Text(self.clone().into()))
    }
}