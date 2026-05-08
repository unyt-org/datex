use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    values::{
        core_values::map::{BorrowedMapKey, Map},
        value_container::ValueContainer,
    },
};
use serde::{
    Serialize, Serializer,
    de::{IgnoredAny, MapAccess},
    ser::SerializeMap,
};

impl<'a> Serialize for BorrowedMapKey<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            BorrowedMapKey::Text(s) => serializer.serialize_str(s),
            BorrowedMapKey::Value(v) => v.serialize(serializer),
        }
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.size()))?;

        for (key, value) in self.iter() {
            map.serialize_entry(&key, value)?;
        }

        map.end()
    }
}

use core::fmt;
use serde::de::{DeserializeSeed, Error, SeqAccess, Visitor};

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
