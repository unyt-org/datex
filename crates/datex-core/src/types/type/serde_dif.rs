use crate::{
    dif::serde_context::SerdeContext, types::r#type::Type,
    utils::serde_serialize_seed::SerializeSeed,
};
use serde::Serializer;

impl<'ctx> SerializeSeed for SerdeContext<'ctx, Type> {
    type Value = Type;

    fn serialize<S>(
        &mut self,
        _value: &Self::Value,
        _serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}
