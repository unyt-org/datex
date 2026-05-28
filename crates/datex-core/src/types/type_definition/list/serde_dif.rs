use crate::{
    dif::serde_context::SerdeContext,
    types::{r#type::Type, type_definition::list::ListTypeDefinition},
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{
    Serializer,
    de::{DeserializeSeed, Visitor},
    ser::SerializeSeq,
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ListTypeDefinition> {
    type Value = ListTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for item in value.iter() {
            seq.serialize_element(&ValueWithSeed::new(
                item,
                self.cast::<Type>(),
            ))?;
        }
        seq.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, ListTypeDefinition>
{
    type Value = ListTypeDefinition;

    fn deserialize<D: serde::de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, ListTypeDefinition> {
    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("a list of type definitions")
    }
    type Value = ListTypeDefinition;

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
