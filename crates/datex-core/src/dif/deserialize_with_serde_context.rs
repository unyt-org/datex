use serde::Deserializer;
use crate::dif::serde_context::SerdeContext;

/// A trait for types that can be deserialized with a `SerdeContext`.
pub trait DeserializeWithSerdeContext<'de>: Sized {
    fn deserialize_with_ctx<D: Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error>;
}