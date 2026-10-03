use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::SerializeSeed,
};
use core::fmt::Display;
use serde::{Deserializer, Serializer, de::DeserializeSeed};

use crate::{prelude::*, types::r#type::Type};
use crate::dif::serde_context::DeserializeSerdeContext;
use crate::utils::serde_serialize_seed::DeserializeWithSerdeContext;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ListCollectionTypeDefinition(pub Box<Type>);
impl Display for ListCollectionTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "[{}]", self.0)
    }
}
impl ListCollectionTypeDefinition {
    pub fn new(item: Type) -> Self {
        Self(Box::new(item))
    }
}
impl<'ctx> SerializeSeed for ListCollectionTypeDefinition {
    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        self.0.serialize_seed(ctx, serializer)
    }
}

/// Deserialization implementations for [ListCollectionTypeDefinition].
impl<'de> DeserializeWithSerdeContext<'de> for ListCollectionTypeDefinition
{
    fn deserialize_with_ctx<D>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let item_type = DeserializeSerdeContext::new(ctx).cast::<Type>().deserialize(deserializer)?;
        Ok(ListCollectionTypeDefinition(Box::new(item_type)))
    }
}
