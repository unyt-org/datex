use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSerdeContext},
    values::{
        core_values::map::{BorrowedMapKey, Map, MapEntries},
        value_container::ValueContainer,
    },
};
use core::fmt;
use indexmap::IndexMap;
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
};

impl<'ctx> SerializeSeed for BorrowedMapKey<'ctx> {
    fn serialize_seed<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            BorrowedMapKey::Text(s) => serializer.serialize_str(s),
            BorrowedMapKey::Value(v) => {
                v.serialize_seed(ctx, serializer)
            }
        }
    }
}

impl<'ctx> SerializeSeed
    for (ValueContainer, ValueContainer)
{
    fn serialize_seed<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;

        tuple.serialize_element(&ValueWithSerdeContext::new(
            &self.0,
            ctx,
        ))?;

        tuple.serialize_element(&ValueWithSerdeContext::new(
            &self.1,
            ctx,
        ))?;

        tuple.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, (ValueContainer, ValueContainer)>
{
    type Value = (ValueContainer, ValueContainer);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, self)
    }
}

impl<'de, 'ctx> Visitor<'de>
    for SerdeContext<'ctx, (ValueContainer, ValueContainer)>
{
    type Value = (ValueContainer, ValueContainer);

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a map entry tuple [key, value]")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let key = seq
            .next_element_seed(self.cast::<ValueContainer>())?
            .ok_or_else(|| de::Error::custom("missing map entry key"))?;

        let value = seq
            .next_element_seed(self.cast::<ValueContainer>())?
            .ok_or_else(|| de::Error::custom("missing map entry value"))?;

        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(de::Error::custom(
                "expected map entry tuple with exactly 2 elements",
            ));
        }

        Ok((key, value))
    }
}

impl<'ctx> SerializeSeed
    for Vec<(ValueContainer, ValueContainer)>
{
    fn serialize_seed<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for entry in self {
            seq.serialize_element(&ValueWithSerdeContext::new(
                entry,
                ctx,
            ))?;
        }

        seq.end()
    }
}

impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, Vec<(ValueContainer, ValueContainer)>>
{
    type Value = Vec<(ValueContainer, ValueContainer)>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de>
    for SerdeContext<'ctx, Vec<(ValueContainer, ValueContainer)>>
{
    type Value = Vec<(ValueContainer, ValueContainer)>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a sequence of map entry tuples")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut entries = Vec::new();

        while let Some(entry) = seq.next_element_seed(
            self.cast::<(ValueContainer, ValueContainer)>(),
        )? {
            entries.push(entry);
        }

        Ok(entries)
    }
}

impl<'ctx> SerializeSeed for Map {
    fn serialize_seed<S>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.entries {
            MapEntries::StructuralWithStringKeys(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;

                for (key, value) in entries {
                    map.serialize_key(key)?;
                    map.serialize_value(&ValueWithSerdeContext::new(
                        value,
                        ctx,
                    ))?;
                }

                map.end()
            }

            MapEntries::Structural(entries) => entries
                .serialize_seed(ctx, serializer),

            MapEntries::Dynamic(entries) => {
                let mut seq = serializer.serialize_seq(Some(entries.len()))?;

                for (key, value) in entries {
                    let entry = (key.clone(), value.clone());

                    seq.serialize_element(&ValueWithSerdeContext::new(
                        &entry,
                        ctx,
                    ))?;
                }

                seq.end()
            }
        }
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Map> {
    type Value = Map;

    fn deserialize<D>(self, deserializer: D) -> Result<Map, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Map> {
    type Value = Map;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            "either an object with string keys or a sequence of [key, value] entries",
        )
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Map, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries = Vec::new();

        while let Some(key) = map.next_key::<String>()? {
            let value = map.next_value_seed(self.cast::<ValueContainer>())?;

            entries.push((key, value));
        }

        Ok(MapEntries::StructuralWithStringKeys(entries).into())
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Map, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut entries = Vec::new();

        while let Some(entry) = seq.next_element_seed(
            self.cast::<(ValueContainer, ValueContainer)>(),
        )? {
            entries.push(entry);
        }

        Ok(MapEntries::Dynamic(IndexMap::from_iter(entries)).into())
    }
}
