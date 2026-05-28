use core::fmt;

use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::SerializeSeq,
};

use crate::{
    dif::serde_context::SerdeContext,
    types::type_definition::{
        TypeDefinition, tagged_type::TaggedTypeDefinition,
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, TaggedTypeDefinition> {
    type Value = TaggedTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&value.tag)?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.ty,
            self.cast::<Option<Box<TypeDefinition>>>(),
        ))?;
        seq.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Option<Box<TypeDefinition>>> {
    type Value = Option<Box<TypeDefinition>>;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(ty) => {
                let mut seed = self.cast::<TypeDefinition>();
                seed.serialize(ty, serializer)
            }
            None => serializer.serialize_none(),
        }
    }
}

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, TaggedTypeDefinition>
{
    type Value = TaggedTypeDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, TaggedTypeDefinition> {
    type Value = TaggedTypeDefinition;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a tuple [tag, ty]")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tag = seq
            .next_element()?
            .ok_or_else(|| de::Error::custom("expected tag"))?;

        let ty = seq
            .next_element_seed(self.cast::<Option<Box<TypeDefinition>>>())?
            .ok_or_else(|| de::Error::custom("expected ty"))?;

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::custom("expected exactly 2 elements"));
        }

        Ok(TaggedTypeDefinition { tag, ty })
    }
}

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, Option<Box<TypeDefinition>>>
{
    type Value = Option<Box<TypeDefinition>>;

    fn deserialize<D>(
        mut self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de, 'ctx> Visitor<'de>
    for SerdeContext<'ctx, Option<Box<TypeDefinition>>>
{
    type Value = Option<Box<TypeDefinition>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an optional TypeDefinition")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ty = self.cast::<TypeDefinition>().deserialize(deserializer)?;

        Ok(Some(Box::new(ty)))
    }
}
