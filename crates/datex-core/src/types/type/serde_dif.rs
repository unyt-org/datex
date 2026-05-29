use crate::{
    dif::serde_context::SerdeContext, types::r#type::Type,
    utils::serde_serialize_seed::SerializeSeed,
};
use serde::{
    Serializer,
    de::{DeserializeSeed, Visitor},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Type> {
    type Value = Type;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            // Type::Alias()
        }
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Type> {
    type Value = Type;

    fn deserialize<D: serde::de::Deserializer<'de>>(
        self,
        _deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        todo!()
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Type> {
    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str("a type definition")
    }
    type Value = Type;
}
