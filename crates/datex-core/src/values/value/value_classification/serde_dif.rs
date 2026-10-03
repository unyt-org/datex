use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    shared_values::PointerAddress,
    types::entity_type::EntityType,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
    values::value::value_classification::{ValueClassification, ValueTag},
};
use serde::{
    Deserializer, Serializer,
    de::{DeserializeSeed, SeqAccess, Visitor},
    ser::SerializeSeq,
};
use crate::dif::serde_context::DeserializeSerdeContext;
use crate::utils::serde_serialize_seed::DeserializeWithSerdeContext;

/// Serialization for [ValueClassification].
impl<'ctx> SerializeSeed for ValueClassification {
    fn serialize_seed<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;

        todo!();
        // seq.serialize_element(value.as_ref())?;
        //
        // match value {
        //     ValueClassification::new_unclassified() => {}
        //     ValueClassification::Entity(entity_type) => {
        //         seq.serialize_element(&ValueWithSeed::new(
        //             entity_type,
        //             self.cast::<EntityType>(),
        //         ))?;
        //     }
        //     ValueClassification::Impls(impls) => {
        //         seq.serialize_element(impls)?;
        //     }
        //     ValueClassification::Tag(tag) => {
        //         seq.serialize_element(&tag.tag)?;
        //         seq.serialize_element(&tag.is_empty)?;
        //     }
        // };

        seq.end()
    }
}

impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for ValueClassification
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

    fn visit_seq<A>(mut self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        todo!();
        // let mut seq = seq;
        // let classification: String = seq.next_element()?.ok_or_else(|| {
        //     serde::de::Error::custom("expected a value classification")
        // })?;
        //
        // match classification.as_str() {
        //     "None" => Ok(ValueClassification::new_unclassified()),
        //     "Entity" => {
        //         let entity_type: EntityType = seq
        //             .next_element_seed(self.cast::<EntityType>())?
        //             .ok_or_else(|| {
        //                 serde::de::Error::custom("expected an entity type")
        //             })?;
        //         Ok(ValueClassification::Entity(entity_type))
        //     }
        //     "Impls" => {
        //         let impls: Vec<PointerAddress> =
        //             seq.next_element()?.ok_or_else(|| {
        //                 serde::de::Error::custom("expected a list of impls")
        //             })?;
        //         Ok(ValueClassification::Impls(impls))
        //     }
        //     "Tag" => {
        //         let tag: String = seq.next_element()?.ok_or_else(|| {
        //             serde::de::Error::custom("expected a tag string")
        //         })?;
        //         let is_empty: bool = seq.next_element()?.ok_or_else(|| {
        //             serde::de::Error::custom("expected a boolean for is_empty")
        //         })?;
        //         Ok(ValueClassification::Tag(ValueTag { tag, is_empty }))
        //     }
        //     _ => Err(serde::de::Error::custom(format!(
        //         "unknown value classification: {}",
        //         classification
        //     ))),
        // }
    }
}
