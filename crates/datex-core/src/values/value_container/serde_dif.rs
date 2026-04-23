use crate::{
    dif::pointer_address::PointerAddressWithOwnership,
    prelude::*,
    shared_values::{
        ReferenceMutability, SharedContainer, SharedContainerOwnership,
    },
    values::{
        core_value::CoreValue, core_values::integer::Integer, value::Value,
        value_container::ValueContainer,
    },
};
use serde::{
    Deserialize, Serialize, Serializer, de::IntoDeserializer,
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

use crate::{
    dif::deserialization_context::DeserializationContext, prelude::*,
    shared_values::PointerAddress,
};
use alloc::format;
use core::fmt;
use serde::{
    Deserializer,
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
/// Deserialization for [ValueContainer] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de>
    for DeserializationContext<'ctx, ValueContainer>
{
    type Value = ValueContainer;
    fn deserialize<D: Deserializer<'de>>(
        self,
        d: D,
    ) -> Result<ValueContainer, D::Error> {
        d.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializationContext<'ctx, ValueContainer> {
    type Value = ValueContainer;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a pointer address string or a Value map")
    }

    /// Visits a pointer address with ownership information encoded as string:
    /// "'$ABCDEF" | "'mut$ABCDEF" | "$ABCDEF"
    fn visit_str<E: serde::de::Error>(
        mut self,
        v: &str,
    ) -> Result<ValueContainer, E> {
        Ok(ValueContainer::Shared(
            self.cast::<SharedContainer>()
                .deserialize(v.into_deserializer())?,
        ))
    }

    // map => local Value
    fn visit_map<A: MapAccess<'de>>(
        mut self,
        map: A,
    ) -> Result<ValueContainer, A::Error> {
        // reuse the Value visitor directly
        let value = self.cast::<Value>().visit_map(map)?;
        Ok(ValueContainer::Local(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dif::cache::DIFSharedContainerCache,
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        runtime::memory::Memory,
        values::{
            core_value::CoreValue,
            core_values::{integer::Integer, list::List},
        },
    };

    fn deserialize_json_string(
        str: impl Into<String>,
        dif_cache: &mut DIFSharedContainerCache,
    ) -> ValueContainer {
        DeserializationContext::<ValueContainer>::new(dif_cache)
            .deserialize(&mut serde_json::Deserializer::from_str(
                str.into().as_str(),
            ))
            .unwrap()
    }

    #[test]
    fn value_container() {
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
    fn deserialize_nested_pointer_address_to_shared_container() {
        let json = r#"{"value": ["'$030000"]}"#; // [integer]

        let memory = Memory::new();
        let dif_cache = &mut DIFSharedContainerCache::default();

        let integer_container = SharedContainer::Referenced(
            memory
                .get_core_reference(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Integer,
                ))
                .clone(),
        );
        dif_cache.store_shared_container(integer_container.clone());

        let outer = deserialize_json_string(json, dif_cache);

        assert_eq!(
            outer,
            ValueContainer::Local(Value::from(CoreValue::List(List::from(
                vec![ValueContainer::Shared(integer_container)]
            ))))
        );
    }
}
