use core::fmt;

use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::map::MapTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeTuple},
};

/// Serde implementations for [MapTypeDefinition].
impl<'ctx> SerializeSeed for SerdeContext<'ctx, MapTypeDefinition> {
    type Value = MapTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;

        for (key, value) in value.iter() {
            seq.serialize_element(&ValueWithSeed::new(
                &(key.clone(), value.clone()),
                self.cast::<(Type, Type)>(),
            ))?;
        }

        seq.end()
    }
}

/// Serde implementations for inner tuple type `(Type, Type)`.
impl<'ctx> SerializeSeed for SerdeContext<'ctx, (Type, Type)> {
    type Value = (Type, Type);

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;

        tuple.serialize_element(&ValueWithSeed::new(
            &value.0,
            self.cast::<Type>(),
        ))?;

        tuple.serialize_element(&ValueWithSeed::new(
            &value.1,
            self.cast::<Type>(),
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
        let mut result = MapTypeDefinition::new();

        while let Some((key, value)) =
            seq.next_element_seed(self.cast::<(Type, Type)>())?
        {
            result.0.push((key, value));
        }

        Ok(result)
    }
}
