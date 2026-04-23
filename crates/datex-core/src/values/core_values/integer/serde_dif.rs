use crate::{
    prelude::*,
    shared_values::{
        ReferenceMutability, SharedContainer, SharedContainerOwnership,
    },
    values::{
        core_value::CoreValue, core_values::integer::Integer, value::Value,
        value_container::ValueContainer,
    },
};
use alloc::format;
use serde::{Serialize, Serializer, ser::SerializeStruct};

impl Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}
