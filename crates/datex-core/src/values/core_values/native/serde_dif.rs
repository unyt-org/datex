use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::SerializeSeed,
    values::core_values::native::NativeCoreValue,
};
use serde::Serializer;

/// Serialization for [NativeCoreValue].
impl<'ctx> SerializeSeed for SerdeContext<'ctx, NativeCoreValue> {
    type Value = NativeCoreValue;

    fn serialize<S: Serializer>(
        &mut self,
        _value: &Self::Value,
        _serializer: S,
    ) -> Result<S::Ok, S::Error> {
        todo!()
        // self.cast::<_>().serialize(value.value.deref(), serializer)
    }
}
