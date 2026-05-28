use core::fmt::Display;

use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{Serializer, de::DeserializeSeed, ser::SerializeSeq};

use crate::types::r#type::Type;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ListSliceCollectionTypeDefinition {
    pub item_type: Box<Type>,
    pub size: usize,
}
impl Display for ListSliceCollectionTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "[{}; {}]", self.item_type, self.size)
    }
}

impl<'ctx> SerializeSeed
    for SerdeContext<'ctx, ListSliceCollectionTypeDefinition>
{
    type Value = ListSliceCollectionTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&ValueWithSeed::new(
            &value.item_type as &Type,
            self.cast::<Type>(),
        ))?;
        seq.serialize_element(&value.size)?;
        seq.end()
    }
}

use core::fmt;
use serde::{
    Deserializer,
    de::{self, SeqAccess, Visitor},
};

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, ListSliceCollectionTypeDefinition>
{
    type Value = ListSliceCollectionTypeDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, self)
    }
}

impl<'de, 'ctx> Visitor<'de>
    for SerdeContext<'ctx, ListSliceCollectionTypeDefinition>
{
    type Value = ListSliceCollectionTypeDefinition;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a tuple [item_type, size]")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let item_type = seq
            .next_element_seed(self.cast::<Type>())?
            .ok_or_else(|| de::Error::custom("expected item type"))?;

        let size = seq
            .next_element()?
            .ok_or_else(|| de::Error::custom("expected size"))?;

        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::custom("expected exactly 2 elements"));
        }

        Ok(ListSliceCollectionTypeDefinition {
            item_type: Box::new(item_type),
            size,
        })
    }
}
