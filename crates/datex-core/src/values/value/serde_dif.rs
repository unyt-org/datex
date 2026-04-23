use crate::{
    prelude::*,
    shared_values::shared_containers::{
        ReferenceMutability, SharedContainer, SharedContainerOwnership,
    },
    values::{
        core_value::CoreValue, core_values::integer::Integer, value::Value,
        value_container::ValueContainer,
    },
};
use alloc::format;
use serde::{Serialize, Serializer, ser::SerializeStruct};

/// Serialization for [Value].
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // serialize as {value: self.inner}
        let mut state = serializer.serialize_struct("Value", 1)?;
        state.serialize_field("value", &self.inner)?;
        state.end()
    }
}
