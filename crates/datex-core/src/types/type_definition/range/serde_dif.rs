use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::range::RangeTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::SerializeTuple,
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, RangeTypeDefinition> {
    type Value = RangeTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&ValueWithSeed::new(
            &*value.start,
            self.cast::<Type>(),
        ))?;
        tuple.serialize_element(&ValueWithSeed::new(
            &*value.end,
            self.cast::<Type>(),
        ))?;
        tuple.end()
    }
}

/// Deserialization implementations for [RangeTypeDefinition].
impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, RangeTypeDefinition>
{
    type Value = RangeTypeDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, RangeTypeDefinition> {
    type Value = RangeTypeDefinition;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter
            .write_str("a tuple of two Type definitions representing a range")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let start = seq
            .next_element_seed(self.cast::<Type>())?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        let end = seq
            .next_element_seed(self.cast::<Type>())?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;

        Ok(RangeTypeDefinition {
            start: Box::new(start),
            end: Box::new(end),
        })
    }
}
