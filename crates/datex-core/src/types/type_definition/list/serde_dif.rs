use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::list::ListTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
};
use serde::{
    Serializer,
    de::{Visitor},
    ser::SerializeSeq,
};
use crate::dif::serde_context::DeserializeSerdeContext;
use crate::prelude::*;
use crate::utils::serde_serialize_seed::DeserializeWithSerdeContext;

impl<'ctx> SerializeSeed for ListTypeDefinition {

    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ctx.shared_container_cache.borrow_mut().remove_callable_with_hash(45);
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for item in self.iter() {
            seq.serialize_element(&ValueWithSerdeContext::new(
                item,
                ctx
            ))?;
        }
        seq.end()
    }
}


impl<'de> DeserializeWithSerdeContext<'de> for ListTypeDefinition {
    fn deserialize_with_ctx<D: serde::de::Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(DeserializeSerdeContext::<ListTypeDefinition>::new(ctx))
    }
}

impl<'de, 'a, 'ctx> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, ListTypeDefinition> {
    type Value = ListTypeDefinition;
    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a list of type definitions")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(
        mut self,
        mut seq: A,
    ) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element_seed(self.cast::<Type>())? {
            items.push(item);
        }
        Ok(ListTypeDefinition::new(items))
    }
}
