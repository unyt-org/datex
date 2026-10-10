use crate::{traits::try_clone::TryClone, values::value::Value};

impl TryClone for Value {
    fn try_clone(&self) -> Result<Value, ()> {
        match self {
            Value::Core(core_value_with_classification) => {
                core_value_with_classification.try_clone()
            }
            Value::Native(native) => native.try_clone(),
        }
    }
}
