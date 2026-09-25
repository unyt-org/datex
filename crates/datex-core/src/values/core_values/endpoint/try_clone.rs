use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::endpoint::Endpoint},
};

impl TryClone for Endpoint {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::Endpoint(self.clone()))
    }
}
