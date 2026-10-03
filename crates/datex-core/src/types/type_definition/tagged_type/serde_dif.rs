use core::fmt;

use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, SeqAccess, Visitor},
    ser::SerializeSeq,
};

use crate::{
    dif::serde_context::SerdeContext,
    types::type_definition::tagged_type::TaggedTypeDefinition,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
};

impl<'ctx> SerializeSeed for TaggedTypeDefinition {
    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.tag)?;
        match &self.ty {
            Some(ty) => {
                seq.serialize_element(&ValueWithSerdeContext::new(
                    ty.as_ref(),
                    ctx,
                ))?;
            }
            None => seq.serialize_element(&ValueWithSerdeContext::new(
                &TypeDefinition::CoreType(CoreLibTypeId::Base(
                    CoreLibBaseTypeId::Unit,
                )),
                ctx,
            ))?,
        }
        seq.end()
    }
}

impl<'ctx> SerializeSeed for Option<Box<Type>> {
    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match self {
            Some(ty) => {
                ty.serialize_seed(ctx, serializer)
            }
            None => serializer.serialize_none(),
        }
    }
}

impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for TaggedTypeDefinition {

    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, DeserializeSerdeContext::new(ctx))
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializeSerdeContext<'de, 'ctx, TaggedTypeDefinition> {
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

use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    prelude::*,
    types::{r#type::Type, type_definition::TypeDefinition},
};
use crate::dif::serde_context::DeserializeSerdeContext;
use crate::utils::serde_serialize_seed::DeserializeWithSerdeContext;

impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for Option<Box<Type>> {

    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(DeserializeSerdeContext::new(ctx))
    }
}

impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for Option<Type> {

    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(DeserializeSerdeContext::new(ctx))
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializeSerdeContext<'de, 'ctx, Option<Type>> {
    type Value = Option<Type>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an optional Type")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
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

        Ok(Some(ty))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }
}

impl<'de, 'ctx> Visitor<'de> for DeserializeSerdeContext<'de, 'ctx, Option<Box<Type>>> {
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

    fn visit_some<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ty = self.cast::<Type>().deserialize(deserializer)?;

        Ok(Some(Box::new(ty)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }
}
