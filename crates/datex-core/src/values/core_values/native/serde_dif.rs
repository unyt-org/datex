use crate::{
    dif::serde_context::SerdeContext,
    dif::serialize_with_serde_context::SerializeWithSerdeContext,
    values::core_values::native::NativeCoreValue,
};
use serde::Serializer;

/// Serialization for [NativeCoreValue].
impl<'ctx> SerializeWithSerdeContext for NativeCoreValue {

    fn serialize_with_ctx<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        todo!()
        // self.cast::<_>().serialize(value.value.deref(), serializer)
    }
}
