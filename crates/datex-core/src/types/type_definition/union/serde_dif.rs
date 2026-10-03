use serde::{Serializer, de::DeserializeSeed, ser::SerializeSeq};

use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::union::UnionTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
};

impl<'ctx> SerializeSeed for UnionTypeDefinition {
    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
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
use crate::dif::serde_context::DeserializeSerdeContext;
use crate::utils::serde_serialize_seed::DeserializeWithSerdeContext;

impl<'de, 'ctx> DeserializeWithSerdeContext<'de> for UnionTypeDefinition {
    fn deserialize_with_ctx<D: Deserializer<'de>>(ctx: &SerdeContext<'_>, deserializer: D) -> Result<Self, D::Error>
    {
        deserializer.deserialize_seq(DeserializeSerdeContext::<UnionTypeDefinition>::new(ctx))
    }
}

impl<'de, 'a, 'ctx> Visitor<'de> for DeserializeSerdeContext<'a, 'ctx, UnionTypeDefinition> {
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
