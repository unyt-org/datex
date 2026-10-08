use crate::{
    traits::try_clone::TryClone,
    values::{
        core_value::CoreValue,
        core_values::integer::typed_integer::TypedInteger,
    },
};
use crate::values::value::Value;

impl TryClone for TypedInteger {
    fn try_clone(&self) -> Result<Value, ()> {
        Ok(CoreValue::TypedInteger(self.clone()).into())
    }
}
