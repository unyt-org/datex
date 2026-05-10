use crate::dif::cache::DIFSharedContainerCache;
use core::marker::PhantomData;

#[derive(Debug)]
pub struct SerdeContext<'ctx, T> {
    pub shared_container_cache: &'ctx mut DIFSharedContainerCache,
    _marker: PhantomData<T>,
}

impl<'ctx, T> SerdeContext<'ctx, T> {
    pub fn new(
        shared_container_cache: &'ctx mut DIFSharedContainerCache,
    ) -> Self {
        Self {
            shared_container_cache,
            _marker: PhantomData,
        }
    }

    /// Converts this deserialization context to a deserialization context for another type U
    pub fn cast<U>(&mut self) -> SerdeContext<'_, U> {
        SerdeContext::new(self.shared_container_cache)
    }

    /// Try to deserialize a JSON string to a DATEX value using the provided context
    #[cfg(test)]
    pub fn try_deserialize_from_json(
        self,
        json_string: &'ctx str,
    ) -> Result<T, serde_json::Error>
    where
        SerdeContext<'ctx, T>: serde::de::DeserializeSeed<'ctx, Value = T>,
    {
        use serde::de::DeserializeSeed;

        DeserializeSeed::deserialize(
            self,
            &mut serde_json::Deserializer::from_str(json_string),
        )
    }

    /// Convert a serializable DATEX value to a JSON string
    #[cfg(test)]
    pub fn serialize_to_json(&mut self, value: &T) -> String
    where
        SerdeContext<'ctx, T>:
            crate::utils::serde_serialize_seed::SerializeSeed<Value = T>,
    {
        use crate::utils::serde_serialize_seed::SerializeSeed;
        let mut serializer = serde_json::Serializer::new(Vec::new());
        self.serialize(&value, &mut serializer).unwrap();
        let bytes = serializer.into_inner();
        String::from_utf8(bytes).unwrap()
    }
}

impl<'ctx> From<&'ctx mut DIFSharedContainerCache> for SerdeContext<'ctx, ()> {
    fn from(cache: &'ctx mut DIFSharedContainerCache) -> Self {
        SerdeContext::new(cache)
    }
}
