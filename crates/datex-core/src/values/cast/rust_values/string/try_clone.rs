use crate::{
    prelude::*, traits::try_clone::TryClone, values::core_value::CoreValue,
};

impl TryClone for String {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::Text(self.clone().into()))
    }
}
