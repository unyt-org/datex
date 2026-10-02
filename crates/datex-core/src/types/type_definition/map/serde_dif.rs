use core::fmt;

use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::map::MapTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
};
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeTuple},
};

/// Serde implementations for [MapTypeDefinition].
impl<'ctx> SerializeSeed for MapTypeDefinition {
    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for (key, value) in self.iter() {
            seq.serialize_element(&ValueWithSerdeContext::new(
                &(key.clone(), value.clone()),
                ctx,
            ))?;
        }

        seq.end()
    }
}

/// Serde implementations for inner tuple type `(Type, Type)`.
impl<'ctx> SerializeSeed for (Type, Type) {
    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;

        tuple.serialize_element(&ValueWithSerdeContext::new(
            &self.0,
            ctx,
        ))?;

        tuple.serialize_element(&ValueWithSerdeContext::new(
            &self.1,
            ctx,
        ))?;

        tuple.end()
    }
}
/// Deserialization implementations for [MapTypeDefinition].
impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, MapTypeDefinition> {
    type Value = MapTypeDefinition;

    fn deserialize<D: serde::de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(self)
    }
}
/// Deserialization implementations for inner tuple type `(Type, Type)`.
impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, (Type, Type)> {
    type Value = (Type, Type);

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_tuple(2, self)
    }
}

/// Visitor implementations for deserialization of inner tuple type `(Type, Type)`.
impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, (Type, Type)> {
    type Value = (Type, Type);

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a type-definition key/value tuple")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let key = seq
            .next_element_seed(self.cast::<Type>())?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        let value = seq
            .next_element_seed(self.cast::<Type>())?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(3, &self));
        }

        Ok((key, value))
    }
}

/// Visitor implementations for deserialization of [MapTypeDefinition].
impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, MapTypeDefinition> {
    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a map type definition")
    }
    type Value = MapTypeDefinition;
    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut result = MapTypeDefinition::default();

        while let Some((key, value)) =
            seq.next_element_seed(self.cast::<(Type, Type)>())?
        {
            result.0.push((key, value));
        }

        Ok(result)
    }
}
