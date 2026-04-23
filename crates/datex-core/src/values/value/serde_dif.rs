use crate::{
    prelude::*,
    values::{core_value::CoreValue, value::Value},
};
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

use crate::dif::deserialization_context::DeserializationContext;
use core::fmt;
use serde::{
    Deserializer,
    de::{DeserializeSeed, MapAccess, Visitor},
};
/// Deserialization for [Value] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for DeserializationContext<'ctx, Value> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // deserialize "value" property as CoreValue
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializationContext<'ctx, Value> {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("struct Value with a 'value' property")
    }

    fn visit_map<A: MapAccess<'de>>(
        mut self,
        mut map: A,
    ) -> Result<Value, A::Error> {
        let mut core_value: Option<CoreValue> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "value" => {
                    core_value =
                        Some(map.next_value_seed(self.cast::<CoreValue>())?);
                }
                _ => {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }

        let core_value = core_value
            .ok_or_else(|| serde::de::Error::missing_field("value"))?;
        Ok(Value {
            inner: core_value,
            custom_type: None,
        })
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
