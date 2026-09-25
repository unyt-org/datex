use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::integer::Integer},
};

impl TryClone for Integer {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::Integer(self.clone()))
    }
}
