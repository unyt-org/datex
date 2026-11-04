use num_traits::ToPrimitive;
use serde::de::Visitor;
use serde::ser::{SerializeSeq, SerializeTuple};
use serde::Serializer;
use crate::dif::deserialize_serde_context::DeserializeSerdeContext;
use crate::dif::deserialize_with_serde_context::{DeserializeWithSerdeContext};
use crate::dif::dif_vec::{DIFVec, OwnedDIFVec};
use crate::dif::serde_context::SerdeContext;
use crate::dif::serialize_with_serde_context::SerializeWithSerdeContext;
use crate::dif::value_with_serde_context::ValueWithSerdeContext;
use crate::libs::core::core_lib_id::CoreLibIdIndex;
use crate::libs::core::type_id::CoreLibBaseTypeId;
use crate::types::entities::entity_impls::EntityImpl;
use crate::types::entities::entity_type_definition::EntityTypeDefinition;
use crate::types::type_definition::TypeDefinition;
use crate::prelude::*;

pub struct InnerEntityTypeDefinition<'a>(&'a EntityTypeDefinition);

impl SerializeWithSerdeContext for EntityTypeDefinition {
    fn serialize_with_ctx<S: Serializer>(&self, ctx: &SerdeContext<'_>, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&CoreLibIdIndex::from(CoreLibBaseTypeId::EntityTypeDefinition).to_u16())?;
        tuple.serialize_element(&ValueWithSerdeContext::new(&InnerEntityTypeDefinition(self), ctx))?;
        tuple.end()
    }
}

impl SerializeWithSerdeContext for InnerEntityTypeDefinition<'_> {
    fn serialize_with_ctx<S: Serializer>(&self, ctx: &SerdeContext<'_>, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq_serializer = serializer.serialize_seq(Some(3))?;

        // name
        seq_serializer.serialize_element(&self.0.name)?;

        // definition
        seq_serializer.serialize_element(&ValueWithSerdeContext::new(
            &self.0.definition,
            ctx,
        ))?;

        // TODO: handle allowed_variants

        // impls
        seq_serializer.serialize_element(&ValueWithSerdeContext::new(
            &DIFVec(&self.0.impls),
            ctx,
        ))?;


        seq_serializer.end()
    }
}


impl<'de> DeserializeWithSerdeContext<'de> for EntityTypeDefinition {
    fn deserialize_with_ctx<D: serde::de::Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.deserialize_any(DeserializeSerdeContext::<EntityTypeDefinition>::new(ctx))
    }
}

impl<'de, 'a, 'ctx> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, EntityTypeDefinition> {
    type Value = EntityTypeDefinition;
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("EntityTypeDefinition")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let name: String = seq.next_element()?.ok_or_else(|| serde::de::Error::custom("Expected name"))?;
        let definition: TypeDefinition = seq.next_element_seed(self.cast())?.ok_or_else(|| serde::de::Error::custom("Expected definition"))?;
        let impls: OwnedDIFVec<EntityImpl> = seq.next_element_seed(self.cast())?.ok_or_else(|| serde::de::Error::custom("Expected impls"))?;

        Ok(EntityTypeDefinition {
            name,
            definition,
            allowed_variants: vec![], // TODO: handle allowed_variants
            impls: impls.0,
            deserialize_from_dif: None,
        })
    }
}