use crate::{
    dif::serde_context::SerdeContext,
    utils::serde_serialize_seed::SerializeSeed,
    values::core_values::native::NativeCoreValue,
};
use serde::Serializer;

/// Serialization for [NativeCoreValue].
impl<'ctx> SerializeSeed for NativeCoreValue {

    fn serialize_seed<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        todo!()
        // self.cast::<_>().serialize(value.value.deref(), serializer)
    }
}
