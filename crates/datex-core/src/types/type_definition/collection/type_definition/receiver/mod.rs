use crate::dif::{
    serde_context::SerdeContext,
    serialize_with_serde_context::SerializeWithSerdeContext,
};
use core::fmt::Display;
use serde::{Deserializer, Serializer, de::DeserializeSeed};
mod to_instructions;
use crate::{
    dif::{
        deserialize_serde_context::DeserializeSerdeContext,
        deserialize_with_serde_context::DeserializeWithSerdeContext,
    },
    prelude::*,
    types::r#type::Type,
};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ReceiverTypeDefinition(pub Box<Type>);
impl Display for ReceiverTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "Receiver<{}>", self.0)
    }
}
impl ReceiverTypeDefinition {
    pub fn new(item: Type) -> Self {
        Self(Box::new(item))
    }
}
impl SerializeWithSerdeContext for ReceiverTypeDefinition {
    fn serialize_with_ctx<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        self.0.serialize_with_ctx(ctx, serializer)
    }
}

/// Deserialization implementations for [ReceiverTypeDefinition].
impl<'de> DeserializeWithSerdeContext<'de> for ReceiverTypeDefinition {
    fn deserialize_with_ctx<D>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let item_type = DeserializeSerdeContext::<Type>::new(ctx)
            .deserialize(deserializer)?;
        Ok(ReceiverTypeDefinition(Box::new(item_type)))
    }
}
