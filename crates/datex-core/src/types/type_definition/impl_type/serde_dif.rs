use serde::{Serializer, de::DeserializeSeed, ser::SerializeSeq};

use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::impl_type::ImplTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ImplTypeDefinition> {
    type Value = ImplTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.inner_type as &Type,
            self.cast::<Type>(),
        ))?;
        seq.serialize_element(&value.impl_markers)?;
        seq.end()
    }
}

use core::fmt;
use serde::{
    Deserializer,
    de::{self, SeqAccess, Visitor},
};

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, ImplTypeDefinition>
{
    type Value = ImplTypeDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, ImplTypeDefinition> {
    type Value = ImplTypeDefinition;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a tuple [inner_type, impl_markers]")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let inner_type = seq
            .next_element_seed(self.cast::<Type>())?
            .ok_or_else(|| de::Error::custom("expected inner type"))?;

        let impl_markers = seq
            .next_element()?
            .ok_or_else(|| de::Error::custom("expected impl markers"))?;

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::custom("expected exactly 2 elements"));
        }

        Ok(ImplTypeDefinition {
            inner_type: Box::new(inner_type),
            impl_markers,
        })
    }
}
