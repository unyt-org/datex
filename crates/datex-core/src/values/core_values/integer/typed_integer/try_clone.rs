use crate::{
    traits::try_clone::TryClone,
    values::{
        core_value::CoreValue,
        core_values::integer::typed_integer::TypedInteger,
    },
};

impl TryClone for TypedInteger {
    fn try_clone(&self) -> Result<CoreValue, ()> {
        Ok(CoreValue::TypedInteger(self.clone()))
    }
}
