use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::SerializeSeed,
};
use core::fmt::Display;
use serde::{Deserializer, Serializer, de::DeserializeSeed};

use crate::{prelude::*, types::r#type::Type};

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
impl<'ctx> SerializeSeed for SerdeContext<'ctx, ListCollectionTypeDefinition> {
    type Value = ListCollectionTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seed = self.cast::<Type>();
        seed.serialize(&value.0, serializer)
    }
}

/// Deserialization implementations for [ListCollectionTypeDefinition].
impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, ListCollectionTypeDefinition>
{
    type Value = ListCollectionTypeDefinition;

    fn deserialize<D>(
        mut self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let item_type = self.cast::<Type>().deserialize(deserializer)?;
        Ok(ListCollectionTypeDefinition(Box::new(item_type)))
    }
}
