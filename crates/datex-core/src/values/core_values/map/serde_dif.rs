use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    values::{
        core_values::map::{BorrowedMapKey, Map},
        value_container::ValueContainer,
    },
};
use serde::{
    Serializer,
    de::{IgnoredAny, MapAccess},
    ser::SerializeMap,
};
use crate::utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed};
use core::fmt;
use serde::de::{DeserializeSeed, Error, SeqAccess, Visitor};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, BorrowedMapKey<'ctx>> {
    type Value = BorrowedMapKey<'ctx>;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            BorrowedMapKey::Text(s) => serializer.serialize_str(s),
            BorrowedMapKey::Value(v) => self.cast::<ValueContainer>().serialize(v, serializer),
        }
    }
}


struct MapEntrySeed<'ctx> {
    ctx: SerdeContext<'ctx, ValueContainer>,
}

impl<'de, 'ctx> DeserializeSeed<'de> for MapEntrySeed<'ctx> {
    type Value = (ValueContainer, ValueContainer);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for MapEntrySeed<'ctx> {
    type Value = (ValueContainer, ValueContainer);

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a map entry tuple [key, value]")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let key = seq
            .next_element_seed(self.ctx.cast::<ValueContainer>())?
            .ok_or_else(|| A::Error::custom("missing map entry key"))?;

        let value =
            seq.next_element_seed(self.ctx.cast::<ValueContainer>())?
                .ok_or_else(|| A::Error::custom("missing map entry value"))?;

        Ok((key, value))
    }
}

struct MapEntriesSeed<'ctx> {
    ctx: SerdeContext<'ctx, ValueContainer>,
}

impl<'de, 'ctx> DeserializeSeed<'de> for MapEntriesSeed<'ctx> {
    type Value = Vec<(ValueContainer, ValueContainer)>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for MapEntriesSeed<'ctx> {
    type Value = Vec<(ValueContainer, ValueContainer)>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a list of map entries")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut entries = Vec::new();

        while let Some(entry) = seq.next_element_seed(MapEntrySeed {
            ctx: self.ctx.cast::<ValueContainer>(),
        })? {
            entries.push(entry);
        }

        Ok(entries)
    }
}

impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Map> {
    type Value = Map;

    fn deserialize<D>(self, deserializer: D) -> Result<Map, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Map> {
    type Value = Map;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a map object with `entries`")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Map, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries: Option<Vec<(ValueContainer, ValueContainer)>> = None;
        while let Some(field) = map.next_key::<String>()? {
            match field.as_str() {
                "entries" => {
                    entries = Some(map.next_value_seed(MapEntriesSeed {
                        ctx: self.cast::<ValueContainer>(),
                    })?);
                }

                _ => {
                    let _: IgnoredAny = map.next_value()?;
                }
            }
        }
        let entries = entries
            .ok_or_else(|| A::Error::custom("missing `entries` field"))?;
        Ok(Map::from(entries))
    }
}


impl<'ctx> SerializeSeed for SerdeContext<'ctx, Map> {
    type Value = Map;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(value.size()))?;
        for (key, value) in value.iter() {
            state.serialize_key(
                &ValueWithSeed::new(&key, self.cast::<BorrowedMapKey>()),
            )?;
            state.serialize_value(
                &ValueWithSeed::new(value, self.cast::<ValueContainer>())
            )?;
        }
        state.end()
    }
}

