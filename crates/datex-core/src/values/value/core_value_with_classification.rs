use crate::preludes::derive::{CoreValue, ValueClassification};

#[derive(Debug, Clone)]
pub struct CoreValueWithClassification {
    /// The inner representation of the value, which is a [CoreValue].
    pub inner: CoreValue,
    /// additional extensions for the value, including its entity type, impls, and tag
    pub classification: ValueClassification,
}

impl CoreValueWithClassification {
    pub fn is_uninitialized(&self) -> bool {
        matches!(&self.inner, CoreValue::Uninitialized)
    }

    pub fn is_null(&self) -> bool {
        matches!(self.inner, CoreValue::Null)
    }
}