use serde::{Serializer, de::DeserializeSeed, ser::SerializeSeq};

use crate::{
    dif::serde_context::SerdeContext,
    types::{
        r#type::Type, type_definition::intersection::IntersectionTypeDefinition,
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, IntersectionTypeDefinition> {
    type Value = IntersectionTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for type_def in value.iter() {
            seq.serialize_element(&ValueWithSeed::new(
                type_def,
                self.cast::<Type>(),
            ))?;
        }
        seq.end()
    }
}

use core::fmt;
use serde::{
    Deserializer,
    de::{SeqAccess, Visitor},
};

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, IntersectionTypeDefinition>
{
    type Value = IntersectionTypeDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de>
    for SerdeContext<'ctx, IntersectionTypeDefinition>
{
    type Value = IntersectionTypeDefinition;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("a sequence of types for IntersectionTypeDefinition")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = IntersectionTypeDefinition::default();

        while let Some(type_def) = seq.next_element_seed(self.cast::<Type>())? {
            result.0.push(type_def);
        }

        Ok(result)
    }
}
