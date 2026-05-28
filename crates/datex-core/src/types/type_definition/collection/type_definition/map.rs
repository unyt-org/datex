use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use core::fmt::{self, Display};
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::SerializeSeq,
};

use crate::types::r#type::Type;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct MapCollectionTypeDefinition {
    pub key_type: Box<Type>,
    pub value_type: Box<Type>,
}
impl Display for MapCollectionTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "Map<{}, {}>", self.key_type, self.value_type)
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, MapCollectionTypeDefinition> {
    type Value = MapCollectionTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.key_type as &Type,
            self.cast::<Type>(),
        ))?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.value_type as &Type,
            self.cast::<Type>(),
        ))?;
        seq.end()
    }
}

/// Deserialization implementations for [ListCollectionTypeDefinition].
impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, MapCollectionTypeDefinition>
{
    type Value = MapCollectionTypeDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, self)
    }
}

impl<'de, 'ctx> Visitor<'de>
    for SerdeContext<'ctx, MapCollectionTypeDefinition>
{
    type Value = MapCollectionTypeDefinition;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a tuple")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let key_type = seq
            .next_element_seed(self.cast::<Type>())?
            .ok_or_else(|| de::Error::custom("expected a key type"))?;

        let value_type = seq
            .next_element_seed(self.cast::<Type>())?
            .ok_or_else(|| de::Error::custom("expected a value type"))?;

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::custom("expected exactly 2 elements"));
        }

        Ok(MapCollectionTypeDefinition {
            key_type: Box::new(key_type),
            value_type: Box::new(value_type),
        })
    }
}
