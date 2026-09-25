use crate::{
    traits::try_clone::TryClone,
    values::{core_value::CoreValue, core_values::range::Range},
};

impl TryClone for Range {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::Range(self.clone()))
    }
}
