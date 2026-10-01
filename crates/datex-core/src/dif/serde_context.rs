use crate::runtime::cache::shared_values_cache::SharedValuesCache;
use core::cell::RefCell;
use std::marker::PhantomData;
use crate::utils::serde_serialize_seed::DeserializeWithSerdeContext;

#[derive(Debug)]
pub struct SerdeContext<'ctx> {
    pub shared_container_cache: &'ctx RefCell<SharedValuesCache>,
}

impl<'ctx> SerdeContext<'ctx> {
    pub fn new(shared_container_cache: &'ctx RefCell<SharedValuesCache>) -> Self {
        Self {
            shared_container_cache,
        }
    }

    /// Try to deserialize a JSON string to a DATEX value using the provided context
    #[cfg(test)]
    pub fn try_deserialize_from_json<T>(
        &self,
        json_string: &'ctx str,
    ) -> Result<T, serde_json::Error>
    where
        T: DeserializeWithSerdeContext<'ctx>,
    {
        DeserializeWithSerdeContext::deserialize_with_ctx(
            self,
            &mut serde_json::Deserializer::from_str(json_string),
        )
    }

    /// Convert a serializable DATEX value to a JSON string
    #[cfg(test)]
    pub fn serialize_to_json<T>(&mut self, value: &T) -> String
    where
        T:
            crate::utils::serde_serialize_seed::SerializeSeed
    {
        use crate::{prelude::*};
        let mut serializer = serde_json::Serializer::new(Vec::new());
        value.serialize_seed(self, &mut serializer).unwrap();
        let bytes = serializer.into_inner();
        String::from_utf8(bytes).unwrap()
    }
}



pub struct DeserializeSerdeContext<'a, 'ctx, T> {
    pub(crate) ctx: &'a SerdeContext<'ctx>,
    _marker: PhantomData<T>
}

impl<'a, 'ctx, T> DeserializeSerdeContext<'a, 'ctx, T> {
    pub fn new(ctx: &'a SerdeContext<'ctx>) -> Self {
        Self { ctx, _marker: PhantomData }
    }
    
    pub fn cast<U>(&self) -> DeserializeSerdeContext<'a, 'ctx, U> {
        DeserializeSerdeContext {
            ctx: self.ctx,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for DeserializeSerdeContext<'_, '_, T> {
    fn clone(&self) -> Self { *self }
}
impl<T> Copy for DeserializeSerdeContext<'_, '_, T> {}
