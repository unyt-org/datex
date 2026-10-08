use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::boolean::Boolean},
};
use crate::values::value::Value;

impl TryClone for Boolean {
    fn try_clone(&self) -> Result<Value, ()> {
        Ok(CoreValue::Boolean(self.clone()).into())
    }
}
