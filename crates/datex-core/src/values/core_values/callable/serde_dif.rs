use crate::{
    dif::serde_context::SerdeContext, prelude::*,
    utils::serde_serialize_seed::SerializeSeed,
    values::core_values::callable::Callable,
};
use serde::{Serializer, ser::SerializeTuple};

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
        data.end()

        // todo: also store callable signature information
    }
}
