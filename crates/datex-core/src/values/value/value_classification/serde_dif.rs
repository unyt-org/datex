use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    shared_values::PointerAddress,
    types::entity_type::EntityType,
    dif::serialize_with_serde_context::SerializeWithSerdeContext,
    values::value::value_classification::{ValueClassification, ValueTag},
};
use serde::{de::{DeserializeSeed, SeqAccess, Visitor}, ser::SerializeSeq, Deserialize, Deserializer, Serializer, Serialize};
use crate::dif::deserialize_serde_context::DeserializeSerdeContext;
use crate::dif::deserialize_with_serde_context::DeserializeWithSerdeContext;
use crate::dif::value_with_serde_context::ValueWithSerdeContext;
use crate::preludes::derive::Type;
use crate::shared_values::SharedContainer;

/// Serialization for [ValueClassification].
impl SerializeWithSerdeContext for ValueClassification {
    fn serialize_with_ctx<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;

        // Serialize entity_type if it exists
        if let Some(entity_type) = &self.entity_type {
            seq.serialize_element(
                &ValueWithSerdeContext::new(
                    entity_type.shared_container(),
                    ctx,
                )
            )?;
            if self.impls.is_empty() && self.tag.is_none() {
                return seq.end();
            }
        }
        else {
            seq.serialize_element(&Option::<()>::None)?;
        }

        // Serialize tag if it exists
        if let Some(tag) = &self.tag {
            seq.serialize_element(tag)?;

            if self.impls.is_empty() {
                return seq.end();
            }
        } else {
            seq.serialize_element(&Option::<()>::None)?;
        }

        // Serialize impls if they exist
        if !self.impls.is_empty() {
            seq.serialize_element(&self.impls)?;
        }

        seq.end()
    }
}

impl<'de> DeserializeWithSerdeContext<'de> for ValueClassification
{
    fn deserialize_with_ctx<D>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(DeserializeSerdeContext::<ValueClassification>::new(ctx))
    }
}

impl<'de, 'a, 'ctx> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, ValueClassification> {
    type Value = ValueClassification;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a value classification")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // [entity_type?, [tag: string, is_empty?: true]?, impls[]?]
        let entity_type = seq.next_element_seed(self.cast::<Option<SharedContainer>>())?.flatten()
            .map(|shared| unsafe { EntityType::new_unchecked(shared) });
        let tag: Option<ValueTag> = seq.next_element::<Option<ValueTag>>()?.flatten();
        let impls: Vec<PointerAddress> = seq.next_element::<Option<Vec<PointerAddress>>>()?.flatten().unwrap_or_default();

        Ok(ValueClassification {
            entity_type,
            tag,
            impls,
        })

    }
}


impl Serialize for ValueTag
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(if self.is_empty { Some(2) } else { Some(1) })?;
        seq.serialize_element(&self.tag)?;
        if self.is_empty {
            seq.serialize_element(&true)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for ValueTag
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ValueTagVisitor)
    }
}

struct ValueTagVisitor;

impl<'de, 'a, 'ctx> Visitor<'de> for ValueTagVisitor {
    type Value = ValueTag;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a value tag")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tag: String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Expected a tag string"))?;
        let is_empty: Option<bool> = seq.next_element()?;

        Ok(ValueTag {
            tag,
            is_empty: is_empty.unwrap_or(false),
        })
    }
}