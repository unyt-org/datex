use serde::Serializer;
use crate::dif::serde_context::SerdeContext;
use crate::utils::serde_serialize_seed::SerializeSeed;
use crate::values::core_values::native::NativeCoreValue;

/// Serialization for [NativeCoreValue].
impl<'ctx> SerializeSeed for SerdeContext<'ctx, NativeCoreValue> {
    type Value = NativeCoreValue;

    fn serialize<S: Serializer>(&mut self, value: &Self::Value, serializer: S) -> Result<S::Ok, S::Error> {
        todo!()
        // self.cast::<_>().serialize(value.value.deref(), serializer)
    }
}