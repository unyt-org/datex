use core::fmt;

use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::SerializeSeq,
};

use crate::{
    dif::serde_context::SerdeContext,
    types::type_definition::tagged_type::TaggedTypeDefinition,
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
        match &value.ty {
            Some(ty) => {
                seq.serialize_element(&ValueWithSeed::new(
                    ty.as_ref(),
                    self.cast::<Type>(),
                ))?;
            }
            None => seq.serialize_element(&ValueWithSeed::new(
                &TypeDefinition::CoreType(CoreLibTypeId::Base(CoreLibBaseTypeId::Unit)).into(),
                self.cast::<Type>(),
            ))?,
        }
        seq.end()
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Box<Type>> {
    type Value = Option<Box<Type>>;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(ty) => {
                let mut seed = self.cast::<Type>();
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
            .next_element_seed(self.cast::<Option<Box<Type>>>())?
            .ok_or_else(|| de::Error::custom("expected ty"))?;

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::custom("expected exactly 2 elements"));
        }

        Ok(TaggedTypeDefinition { tag, ty })
    }
}

use crate::{prelude::*, types::r#type::Type};
use crate::libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId};
use crate::types::type_definition::TypeDefinition;

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Option<Box<Type>>> {
    type Value = Option<Box<Type>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Option<Box<Type>>> {
    type Value = Option<Box<Type>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an optional Type")
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
        let ty = self.cast::<Type>().deserialize(deserializer)?;

        Ok(Some(Box::new(ty)))
    }
}
