use crate::{
    traits::try_clone::TryClone,
    values::{
        core_value::CoreValue,
        core_values::{endpoint::Endpoint, instant::Instant},
        value::Value,
    },
};

impl TryClone for Instant {
    fn try_clone(&self) -> Result<Value, ()> {
        Ok(CoreValue::Instant(self.clone()).into())
    }
}
