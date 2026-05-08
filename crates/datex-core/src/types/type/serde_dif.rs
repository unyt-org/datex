use serde::Serializer;
use crate::dif::serde_context::SerdeContext;
use crate::types::r#type::Type;
use crate::utils::serde_serialize_seed::{SerializeSeed};

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Type> {
    type Value = Type;

    fn serialize<S>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}
