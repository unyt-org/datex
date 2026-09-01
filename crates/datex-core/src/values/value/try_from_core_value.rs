use crate::values::core_value::CoreValue;
use crate::values::value::Value;
use crate::values::value::value_classification::ValueClassification;

impl TryFrom<CoreValue> for Value {
    type Error = ();
    fn try_from(value: CoreValue) -> Result<Self, Self::Error> {
        Ok(Value::new(value, ValueClassification::default()))
    }
}