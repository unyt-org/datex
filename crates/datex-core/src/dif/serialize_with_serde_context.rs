use serde::Serializer;
use crate::dif::serde_context::SerdeContext;

/// A trait for types that can be serialized with a `SerdeContext`.
pub trait SerializeWithSerdeContext {
    fn serialize_with_ctx<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
}

impl<T: SerializeWithSerdeContext + ?Sized> SerializeWithSerdeContext for &T {
    fn serialize_with_ctx<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        (**self).serialize_with_ctx(ctx, serializer)
    }
}

impl<T: SerializeWithSerdeContext + ?Sized> SerializeWithSerdeContext for &mut T {
    fn serialize_with_ctx<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        (**self).serialize_with_ctx(ctx, serializer)
    }
}
