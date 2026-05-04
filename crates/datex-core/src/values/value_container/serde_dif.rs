use crate::{
    shared_values::SharedContainer,
    values::{value::Value, value_container::ValueContainer},
};
use serde::{Deserialize, Serialize, Serializer, de::IntoDeserializer};

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

use crate::dif::serde_context::SerdeContext;
use core::fmt;
use serde::{
    Deserializer,
    de::{DeserializeSeed, MapAccess, Visitor},
};
use crate::utils::serde_serialize_seed::SerializeSeed;

/// Deserialization for [ValueContainer] using a [DeserializationContext] to provide access to the memory during deserialization.
impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, ValueContainer> {
    type Value = ValueContainer;
    fn deserialize<D: Deserializer<'de>>(
        self,
        d: D,
    ) -> Result<ValueContainer, D::Error> {
        d.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, ValueContainer> {
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

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ValueContainer> {
    type Value = ValueContainer;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            ValueContainer::Shared(shared) => self
                .cast::<SharedContainer>()
                .serialize(shared, serializer),
            ValueContainer::Local(local) => local.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use log::{info, logger};

    use super::*;
    use crate::{
        dif::cache::DIFSharedContainerCache,
        libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
        prelude::*,
        runtime::{
            memory::Memory,
            pointer_address_provider::SelfOwnedPointerAddressProvider,
        },
        shared_values::{
            OwnedSharedContainer, PointerAddress, SelfOwnedPointerAddress,
            SelfOwnedSharedContainer, SharedContainerMutability,
        },
        values::{
            core_value::CoreValue,
            core_values::{integer::Integer, list::List},
        },
    };
    use core::assert_matches;

    fn deserialize_json_string(
        str: impl Into<String>,
        dif_cache: &mut DIFSharedContainerCache,
    ) -> ValueContainer {
        SerdeContext::<ValueContainer>::new(dif_cache)
            .deserialize(&mut serde_json::Deserializer::from_str(
                str.into().as_str(),
            ))
            .unwrap()
    }

    fn serialize_json_string(
        value: ValueContainer,
        dif_cache: &mut DIFSharedContainerCache,
    ) -> String {
        let mut context = SerdeContext::<ValueContainer>::new(dif_cache);
        let mut serializer = serde_json::Serializer::new(Vec::new());
        context.serialize(&value, &mut serializer).unwrap();
        let bytes = serializer.into_inner();
        String::from_utf8(bytes).unwrap()
    }
    #[test]
    fn owned() {
        let memory = Memory::new();
        let mut provider = SelfOwnedPointerAddressProvider::default();
        let mut cache = DIFSharedContainerCache::default();
        let value = ValueContainer::Shared(SharedContainer::Owned(
            OwnedSharedContainer::new_with_inferred_allowed_type(
                42.into(),
                SharedContainerMutability::Mutable,
                &mut provider,
                &memory,
            ),
        ));
        let serialized = serialize_json_string(value, &mut cache);
        let address_string = serialized
            .replace('"', "")
            .strip_prefix('$')
            .unwrap()
            .to_string();
        let addr = PointerAddress::SelfOwned(
            SelfOwnedPointerAddress::try_from(address_string).unwrap(),
        );
        let container = cache.try_take_owned_shared_container(&addr);
        assert!(container.is_ok());
    }

    #[test]
    fn referenced() {
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
    fn deserialize_nested_pointer_address() {
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
