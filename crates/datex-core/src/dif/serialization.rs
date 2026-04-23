use crate::{
    prelude::*,
    serde::Deserialize,
    shared_values::shared_containers::{
        ReferenceMutability, SharedContainer, SharedContainerOwnership,
    },
    values::{
        core_value::CoreValue, core_values::integer::Integer, value::Value,
        value_container::ValueContainer,
    },
};
use alloc::format;
use serde::{
    Deserializer, Serialize, Serializer, de::DeserializeSeed,
    ser::SerializeStruct,
};

/// Serialization for [ValueContainer].
impl Serialize for ValueContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ValueContainer::Shared(shared) => shared.serialize(serializer),
            ValueContainer::Local(value) => value.serialize(serializer),
        }
    }
}

impl Serialize for SharedContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Only serialize the ownership and pointer address
        let ownership = match self.ownership() {
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Immutable,
            ) => "'",
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Mutable,
            ) => "'mut ",
            SharedContainerOwnership::Owned => "",
        };

        format!("{}{}", ownership, self.pointer_address()).serialize(serializer)
    }
}

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

impl Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
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
        shared_values::shared_containers::SharedContainerMutability,
        values::{core_value::CoreValue, core_values::integer::Integer},
    };

    #[test]
    fn serialize_shared_container_reference() {
        let memory = Memory::new();
        let integer_container =
            ValueContainer::Shared(SharedContainer::Referenced(
                memory
                    .get_core_reference(CoreLibTypeId::Base(
                        CoreLibBaseTypeId::Integer,
                    ))
                    .clone(),
            ));
        let serialized = serde_json::to_string(&integer_container).unwrap();
        assert_eq!(serialized, r#""'$030000""#);
    }

    #[test]
    fn serialize_shared_owned_container() {
        let memory = &Memory::new();
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let owned_container =
            SharedContainer::new_owned_with_inferred_allowed_type(
                ValueContainer::from(42),
                SharedContainerMutability::Mutable,
                address_provider,
                memory,
            );

        let serialized = serde_json::to_string(&owned_container).unwrap();
        assert_eq!(
            serialized,
            format!(r#""{}""#, owned_container.pointer_address().to_string())
        );
    }

    #[test]
    fn serialize_simple_local_value() {
        let value_container = ValueContainer::Local(Value::from(
            CoreValue::Integer(Integer::new(42)),
        ));
        let serialized = serde_json::to_string(&value_container).unwrap();
        assert_eq!(serialized, r#"{"value":"42"}"#);
    }
}
