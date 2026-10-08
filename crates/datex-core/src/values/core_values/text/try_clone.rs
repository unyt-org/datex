use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::text::Text},
};
use crate::values::value::Value;

impl TryClone for Text {
    fn try_clone(&self) -> Result<Value, ()> {
        Ok(CoreValue::Text(self.clone()).into())
    }
}
