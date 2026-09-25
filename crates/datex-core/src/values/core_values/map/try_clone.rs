use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::map::Map},
};

impl TryClone for Map {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::Map(self.clone()))
    }
}
