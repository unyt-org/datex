use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    types::type_definition::{
        collection::{
            CollectionTypeDefinition,
            type_definition::{
                list::ListCollectionTypeDefinition,
                list_slice::ListSliceCollectionTypeDefinition,
                map::MapCollectionTypeDefinition,
            },
        },
        range::RangeTypeDefinition,
    },
    utils::serde_serialize_seed::{SerializeSeed, ValueWithSeed},
};
use serde::{
    Deserializer, Serializer,
    de::{self, DeserializeSeed, Visitor},
    ser::{SerializeMap, SerializeSeq},
};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, CollectionTypeDefinition> {
    type Value = CollectionTypeDefinition;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut obj = serializer.serialize_map(Some(1))?;
        obj.serialize_key(value.as_ref())?;
        match value {
            CollectionTypeDefinition::List(iten) => {
                obj.serialize_value(&ValueWithSeed::new(
                    iten,
                    self.cast::<ListCollectionTypeDefinition>(),
                ))?
            }
            CollectionTypeDefinition::ListSlice(item) => {
                obj.serialize_value(&ValueWithSeed::new(
                    item,
                    self.cast::<ListSliceCollectionTypeDefinition>(),
                ))?
            }
            CollectionTypeDefinition::Map(map) => {
                obj.serialize_value(&ValueWithSeed::new(
                    map,
                    self.cast::<MapCollectionTypeDefinition>(),
                ))?
            }
            CollectionTypeDefinition::Range(range) => obj.serialize_value(
                &ValueWithSeed::new(range, self.cast::<RangeTypeDefinition>()),
            )?,
        }
        obj.end()
    }
}

/// Deserialization implementations for [CollectionTypeDefinition].
impl<'de, 'ctx> DeserializeSeed<'de>
    for SerdeContext<'ctx, CollectionTypeDefinition>
{
    type Value = CollectionTypeDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, CollectionTypeDefinition> {
    type Value = CollectionTypeDefinition;

    fn expecting(
        &self,
        formatter: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        formatter.write_str(
            "a map with a single key representing the collection type",
        )
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let key = map.next_key::<String>()?.ok_or_else(|| {
            de::Error::custom(
                "expected a single key for collection type definition",
            )
        })?;
        match key.as_str() {
            "List" => Ok(CollectionTypeDefinition::List(map.next_value_seed(
                self.cast::<ListCollectionTypeDefinition>(),
            )?)),
            "ListSlice" => {
                Ok(CollectionTypeDefinition::ListSlice(map.next_value_seed(
                    self.cast::<ListSliceCollectionTypeDefinition>(),
                )?))
            }
            "Map" => Ok(CollectionTypeDefinition::Map(map.next_value_seed(
                self.cast::<MapCollectionTypeDefinition>(),
            )?)),
            "Range" => Ok(CollectionTypeDefinition::Range(
                map.next_value_seed(self.cast::<RangeTypeDefinition>())?,
            )),

            _ => Err(de::Error::unknown_variant(
                &key,
                &["List", "ListSlice", "Map", "Range"],
            )),
        }
    }
}
