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

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;
    use crate::{
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        runtime::{
            memory::Memory,
            pointer_address_provider::SelfOwnedPointerAddressProvider,
        },
        shared_values::SharedContainerMutability,
        values::{core_value::CoreValue, core_values::integer::Integer},
    };

    #[test]
    fn serialize_simple_local_value() {
        let value = Value::from(CoreValue::Integer(Integer::new(42)));
        let serialized = serde_json::to_string(&value).unwrap();
        assert_eq!(serialized, r#"{"value":"42"}"#);
    }
}
