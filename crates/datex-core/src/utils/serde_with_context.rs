use serde::{
    Deserialize, Serialize, Serializer, de,
    de::{DeserializeSeed, Deserializer, Error, SeqAccess},
    ser::{SerializeSeq, SerializeTuple},
};
use crate::dif::deserialize_serde_context::DeserializeSerdeContext;
use crate::dif::serde_context::{SerdeContext};
use crate::dif::value_with_serde_context::ValueWithSerdeContext;

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


impl<V: ?Sized + SerializeWithSerdeContext> Serialize for ValueWithSerdeContext<'_, '_, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize_with_ctx(&self.ctx, serializer)
    }
}


/// A trait for types that can be deserialized with a `SerdeContext`.
pub trait DeserializeWithSerdeContext<'de>: Sized {
    fn deserialize_with_ctx<D: Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error>;
}

impl<'de, T: DeserializeWithSerdeContext<'de>> DeserializeSeed<'de> for DeserializeSerdeContext<'_, '_, T>
{
    type Value = T;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<T, D::Error> {
        T::deserialize_with_ctx(self.ctx, deserializer)
    }
}