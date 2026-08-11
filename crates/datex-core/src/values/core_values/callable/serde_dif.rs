use core::fmt;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserializer, Serializer};
use crate::dif::serde_context::SerdeContext;
use crate::utils::serde_serialize_seed::SerializeSeed;
use crate::values::core_values::callable::Callable;
use crate::prelude::*;

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Callable> {
    type Value = Callable;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // put callable into cache
        let hash = self.shared_container_cache.store_callable(value.clone());
        let mut data = serializer.serialize_tuple(1)?;

        // store hash
        data.serialize_element(&hash.to_string())?;

        // store name
        data.serialize_element(&value.name)?;
        data.end()

        // todo: also store function signature information
    }
}


impl<'de, 'ctx> DeserializeSeed<'de> for SerdeContext<'ctx, Callable> {
    type Value = Callable;

    fn deserialize<D>(self, deserializer: D) -> Result<Callable, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Callable> {
    type Value = Callable;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            "either an object with string keys or a sequence of [key, value] entries",
        )
    }
    fn visit_seq<A>(mut self, mut seq: A) -> Result<Callable, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let hash: String = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::custom("Expected hash string as first element"))?;
        let hash_u64 = hash.parse::<u64>().map_err(|_| serde::de::Error::custom("Failed to parse hash string to u64"))?;

        let _name: Option<Option<String>> = seq
            .next_element()?;

        let callable = self.shared_container_cache.get_callable(hash_u64)
            .ok_or_else(|| serde::de::Error::custom(format!("Callable with hash {} not found in cache", hash_u64)))?;

        Ok(callable)
    }
}

