use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::integer::Integer},
};
use crate::values::value::Value;

impl TryClone for Integer {
    fn try_clone(&self) -> Result<Value, ()> {
        Ok(CoreValue::Integer(self.clone()).into())
    }
}
