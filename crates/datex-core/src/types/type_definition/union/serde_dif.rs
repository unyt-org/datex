use serde::{Serializer, de::DeserializeSeed, ser::SerializeSeq};

use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::union::UnionTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
};

impl<'ctx> SerializeSeed for UnionTypeDefinition {
    fn serialize_seed<S: Serializer>(
        &mut self,
        ctx: &SerdeContext<'ctx>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for type_def in self.iter() {
            seq.serialize_element(&ValueWithSerdeContext::new(
                type_def,
                ctx,
            ))?;
        }
        seq.end()
    }
}

use core::fmt;
use serde::{
    Deserializer,
    de::{SeqAccess, Visitor},
};

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, UnionTypeDefinition>
{
    type Value = UnionTypeDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, UnionTypeDefinition> {
    type Value = UnionTypeDefinition;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of types for UnionTypeDefinition")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = UnionTypeDefinition::default();

        while let Some(type_def) = seq.next_element_seed(self.cast::<Type>())? {
            result.0.push(type_def);
        }

        Ok(result)
    }
}
