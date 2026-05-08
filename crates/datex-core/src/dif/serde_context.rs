use crate::dif::cache::DIFSharedContainerCache;
use core::marker::PhantomData;
use serde::de::DeserializeSeed;
use serde::{Deserializer, Serializer};
use crate::runtime::memory::Memory;
use crate::utils::serde_serialize_seed::SerializeSeed;

#[derive(Debug)]
pub struct SerdeContext<'ctx, T> {
    pub shared_container_cache: &'ctx mut DIFSharedContainerCache,
    pub memory: &'ctx mut Memory,
    _marker: PhantomData<T>,
}

impl<'ctx, T> SerdeContext<'ctx, T> {
    pub fn new(
        shared_container_cache: &'ctx mut DIFSharedContainerCache,
        memory: &'ctx mut Memory,
    ) -> Self {
        Self {
            shared_container_cache,
            memory,
            _marker: PhantomData,
        }
    }

    /// Converts this deserialization context to a deserialization context for another type U
    pub fn cast<U>(&mut self) -> SerdeContext<'_, U> {
        SerdeContext::new(self.shared_container_cache, self.memory)
    }

    /// Try to deserialize a JSON string to a DATEX value using the provided context
    #[cfg(test)]
    pub fn try_deserialize_from_json(
        self,
        json_string: &'ctx str
    ) -> Result<T, serde_json::Error>
    where
        SerdeContext<'ctx, T>: DeserializeSeed<'ctx, Value = T>,
    {
        DeserializeSeed::deserialize(
            self,
            &mut serde_json::Deserializer::from_str(
                json_string,
            ),
        )
    }

    /// Convert a serializable DATEX value to a JSON string
    #[cfg(test)]
    pub fn serialize_to_json(
        &mut self,
        value: &T,
    ) -> String
    where
        SerdeContext<'ctx, T>: SerializeSeed<Value = T>,
    {
        let mut serializer = serde_json::Serializer::new(Vec::new());
        self.serialize(&value, &mut serializer).unwrap();
        let bytes = serializer.into_inner();
        String::from_utf8(bytes).unwrap()
    }

}