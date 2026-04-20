use core::fmt;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserializer;
use crate::serde::deserialization_context::DeserializationContext;
use crate::serde::Deserialize;
use crate::shared_values::pointer_address::PointerAddress;
use crate::shared_values::shared_containers::SharedContainer;
use crate::values::core_value::CoreValue;
use crate::values::core_values::list::List;
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

/// Deserialization for [ValueContainer] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for DeserializationContext<'ctx, ValueContainer> {
    type Value = ValueContainer;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<ValueContainer, D::Error> {
        d.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializationContext<'ctx, ValueContainer> {
    type Value = ValueContainer;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a pointer address string or a Value map")
    }

    // string => pointer address
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<ValueContainer, E> {
        let address = PointerAddress::try_from(v)
            .map_err(|_| E::custom(format!("invalid pointer address: {}", v)))?;
        let reference = self.memory.get_reference(&address)
            .ok_or_else(|| E::custom(format!("pointer address {} not found in memory", v)))?;
        Ok(ValueContainer::Shared(SharedContainer::Referenced(reference.clone())))
    }

    // map => local Value
    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<ValueContainer, A::Error> {
        // reuse the Value visitor directly
        let value = self.cast::<Value>().visit_map(map)?;
        Ok(ValueContainer::Local(value))
    }
}


/// Deserialization for [Value] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for DeserializationContext<'ctx, Value> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>
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

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut core_value: Option<CoreValue> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "value" => {
                    core_value = Some(map.next_value_seed(self.cast::<CoreValue>())?);
                }
                _ => { map.next_value::<serde::de::IgnoredAny>()?; }
            }
        }

        let core_value = core_value.ok_or_else(|| serde::de::Error::missing_field("value"))?;
        Ok(Value { inner: core_value, custom_type: None })
    }
}


/// Deserialization for [CoreValue] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for DeserializationContext<'ctx, CoreValue> {
    type Value = CoreValue;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<CoreValue, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializationContext<'ctx, CoreValue> {
    type Value = CoreValue;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a CoreValue")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<CoreValue, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element_seed(self.cast::<ValueContainer>())? {
            items.push(item);
        }
        Ok(CoreValue::List(List::from(items)))
    }
}

#[cfg(test)]
mod tests {
    use crate::libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId};
    use crate::runtime::memory::Memory;
    use crate::values::core_value::CoreValue;
    use crate::values::core_values::list::List;
    use super::*;

    #[test]
    fn deserialize_pointer_address_to_shared_container() {
        let json = r#""030000""#; // integer

        let memory = Memory::new();

        let outer = DeserializationContext::<ValueContainer>::new(&memory)
            .deserialize(&mut serde_json::Deserializer::from_str(json))
            .unwrap();

        println!("{:#?}", outer);

        assert_eq!(
            outer,
            ValueContainer::Shared(SharedContainer::Referenced(memory.get_core_reference(CoreLibTypeId::Base(CoreLibBaseTypeId::Integer)).clone()))
        );
    }

    #[test]
    fn deserialize_nested_pointer_address_to_shared_container() {
        let json = r#"{"value": ["030000"]}"#; // [integer]

        let memory = Memory::new();

        let outer = DeserializationContext::<ValueContainer>::new(&memory)
            .deserialize(&mut serde_json::Deserializer::from_str(json))
            .unwrap();
        
        assert_eq!(
            outer,
            ValueContainer::Local(
                Value::from(CoreValue::List(List::from(vec![
                    ValueContainer::Shared(SharedContainer::Referenced(memory.get_core_reference(CoreLibTypeId::Base(CoreLibBaseTypeId::Integer)).clone()))
                ])))
            )
        );
    }
}