use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::list::List},
};

impl TryClone for List {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::List(self.clone()))
    }
}
