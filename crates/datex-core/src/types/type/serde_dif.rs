use core::ops::Deref;
use crate::{
    dif::serde_context::SerdeContext, types::r#type::Type,
    utils::serde_serialize_seed::SerializeSeed,
};
use serde::{
    Serializer,
    de::{DeserializeSeed, Visitor},
};
use serde::de::MapAccess;
use crate::shared_values::SharedContainer;
use crate::types::shared_container_containing_nominal_type::SharedContainerContainingNominalType;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_definition_with_metadata::TypeDefinitionWithMetadata;
use crate::values::value_container::serde_dif::SHARED_CONTAINER_KEY;

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Type> {
    type Value = Type;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Type::Alias(type_definition) =>
                self.cast::<TypeDefinitionWithMetadata>().serialize(type_definition, serializer),
            Type::Nominal(shared_container_containing_nominal_type) =>
                self.cast::<SharedContainer>().serialize(shared_container_containing_nominal_type.deref(), serializer),
        }
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Type> {
    type Value = Type;

    fn deserialize<D: serde::de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Type> {
    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a type definition")
    }
    type Value = Type;


    fn visit_seq<A>(
        mut self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let metadata = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("missing metadata"))?;
        let definition = seq
            .next_element_seed(self.cast::<TypeDefinition>())?
            .ok_or_else(|| serde::de::Error::custom("missing definition"))?;
        Ok(Type::Alias(TypeDefinitionWithMetadata { metadata, definition }))
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let Some(key) = map.next_key::<String>()? else {
            return Err(serde::de::Error::custom(
                "expected shared container map with '$' as key",
            ));
        };

        if key != SHARED_CONTAINER_KEY {
            return Err(serde::de::Error::custom(format!(
                "unexpected key {key:?} in shared container map, expected '{SHARED_CONTAINER_KEY}'"
            )));
        }

        let shared = map.next_value_seed(self.cast::<SharedContainer>())?;

        if let Some(extra_key) = map.next_key::<String>()? {
            return Err(serde::de::Error::custom(format!(
                "unexpected extra key {extra_key:?} in shared container map, expected only '{SHARED_CONTAINER_KEY}'"
            )));
        }

        Ok(Type::Nominal(unsafe {
            SharedContainerContainingNominalType::new_unchecked(shared)
        }))
    }

}
