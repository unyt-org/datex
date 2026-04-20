use serde::de::DeserializeSeed;
use serde::{Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;
use crate::serde::Deserialize;
use crate::shared_values::shared_containers::SharedContainer;
use crate::values::core_value::CoreValue;
use crate::values::core_values::integer::Integer;
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

/// Serialization for [ValueContainer].
impl Serialize for ValueContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
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
        S: Serializer
    {
        // Only serialize the pointer address
        self.pointer_address().serialize(serializer)
    }
}


/// Serialization for [Value].
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
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
        S: Serializer
    {
        match self {
            CoreValue::Integer(i) => i.serialize(serializer),
            _ => todo!()
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
    use crate::libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId};
    use crate::runtime::memory::Memory;
    use crate::values::core_value::CoreValue;
    use crate::values::core_values::integer::Integer;
    use super::*;

    #[test]
    fn serialize_shared_container() {
        let memory = Memory::new();
        let integer_container = ValueContainer::Shared(SharedContainer::Referenced(memory.get_core_reference(CoreLibTypeId::Base(CoreLibBaseTypeId::Integer)).clone()));
        let serialized = serde_json::to_string(&integer_container).unwrap();
        println!("{}", serialized);
        assert_eq!(serialized, r#""030000""#);
    }

    #[test]
    fn serialize_simple_local_value() {
        let value_container = ValueContainer::Local(Value::from(CoreValue::Integer(Integer::new(42))));
        let serialized = serde_json::to_string(&value_container).unwrap();
        println!("{}", serialized);
        assert_eq!(serialized, r#"{"value":"42"}"#);
    }
}