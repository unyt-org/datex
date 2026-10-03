use core::fmt;

use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::map::MapTypeDefinition},
    dif::serialize_with_serde_context::SerializeWithSerdeContext,
};
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::{SerializeSeq, SerializeTuple},
};
use crate::dif::deserialize_serde_context::DeserializeSerdeContext;
use crate::dif::value_with_serde_context::ValueWithSerdeContext;
use crate::dif::deserialize_with_serde_context::DeserializeWithSerdeContext;

/// Serde implementations for [MapTypeDefinition].
impl<'ctx> SerializeWithSerdeContext for MapTypeDefinition {
    fn serialize_with_ctx<S: Serializer>(
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
impl<'ctx> SerializeWithSerdeContext for (Type, Type) {
    fn serialize_with_ctx<S: Serializer>(
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
impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for MapTypeDefinition {
    fn deserialize_with_ctx<D: Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(DeserializeSerdeContext::<MapTypeDefinition>::new(ctx))
    }
}
/// Deserialization implementations for inner tuple type `(Type, Type)`.
impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for (Type, Type) {
    fn deserialize_with_ctx<D: Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.deserialize_tuple(2, DeserializeSerdeContext::<(Type, Type)>::new(ctx))
    }
}

/// Visitor implementations for deserialization of inner tuple type `(Type, Type)`.
impl<'de, 'a, 'ctx> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, (Type, Type)> {
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
impl<'de, 'a, 'ctx> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, MapTypeDefinition> {
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
