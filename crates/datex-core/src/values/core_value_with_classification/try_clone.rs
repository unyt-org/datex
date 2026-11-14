use crate::{
    traits::try_clone::TryClone,
    values::{
        core_value_with_classification::CoreValueWithClassification,
        value::Value,
    },
};

impl TryClone for CoreValueWithClassification {
    /// Tries to clone the [CoreValueWithClassification] into a [Value::Core] variant containing the cloned value.
    /// The result is always Ok, as the CoreValue::TryClone implementation for the inner value always succeeds and gives
    /// back a cloned [CoreValueWithClassification] wrapped in a [Value::Core] variant.
    fn try_clone(&self) -> Result<Value, ()> {
        let core_value = self.inner.try_clone()?;
        match core_value {
            Value::Core(core_value_with_classification) => {
                Ok(Value::Core(CoreValueWithClassification {
                    inner: core_value_with_classification.inner,
                    classification: self.classification.clone(),
                }))
            }
            _ => unreachable!(),
        }
    }
}
