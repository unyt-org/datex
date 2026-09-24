use crate::traits::try_clone::TryClone;
use crate::values::core_value::CoreValue;
use crate::values::core_values::endpoint::Endpoint;

impl TryClone for Endpoint {
    fn try_clone(&self) -> Result<CoreValue , ()> {
        Ok(CoreValue::Endpoint(self.clone()))
    }
}