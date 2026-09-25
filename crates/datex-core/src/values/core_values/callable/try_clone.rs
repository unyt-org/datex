use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::callable::Callable},
};

impl TryClone for Callable {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::Callable(self.clone()))
    }
}
