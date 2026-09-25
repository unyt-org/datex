use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::boolean::Boolean},
};

impl TryClone for Boolean {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::Boolean(self.clone()))
    }
}
