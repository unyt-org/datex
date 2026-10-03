use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    types::{r#type::Type, type_definition::range::RangeTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
};
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::SerializeTuple,
};
use crate::dif::serde_context::DeserializeSerdeContext;
use crate::utils::serde_serialize_seed::DeserializeWithSerdeContext;

impl<'ctx> SerializeSeed for RangeTypeDefinition {
    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&ValueWithSerdeContext::new(
            &*self.start,
            ctx,
        ))?;
        tuple.serialize_element(&ValueWithSerdeContext::new(
            &*self.end,
            ctx,
        ))?;
        tuple.end()
    }
}

/// Deserialization implementations for [RangeTypeDefinition].
impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for RangeTypeDefinition {
    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(DeserializeSerdeContext::new(ctx))

    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializeSerdeContext<'de, 'ctx, RangeTypeDefinition> {
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
