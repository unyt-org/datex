use crate::{
    dif::serde_context::SerdeContext, types::type_definition::TypeDefinition,
    utils::serde_serialize_seed::SerializeSeed,
};
use serde::Serializer;

impl<'ctx> SerializeSeed for SerdeContext<'ctx, TypeDefinition> {
    type Value = TypeDefinition;

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
