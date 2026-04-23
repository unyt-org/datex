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

/// Serialization for [CoreValue].
impl Serialize for CoreValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CoreValue::Integer(i) => i.serialize(serializer),
            _ => todo!(),
        }
    }
}
