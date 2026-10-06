use serde::{Serialize, Serializer};
use crate::dif::deserialize_with_serde_context::{DeserializeWithSerdeContext, DeserializeWithSerdeContextDyn};
use crate::dif::serde_context::SerdeContext;
use crate::dif::serialize_with_serde_context::SerializeWithSerdeContextDyn;
use crate::preludes::derive::{SerializeWithSerdeContext};
use crate::prelude::*;

impl<T: SerializeWithSerdeContextDyn> SerializeWithSerdeContext for Box<T> {
    fn serialize_with_ctx<S: Serializer>(&self, ctx: &SerdeContext<'_>, serializer: S) -> Result<S::Ok, S::Error> {
        (*self).with_ctx(ctx).serialize(serializer)
    }
}


impl<'de, T: DeserializeWithSerdeContextDyn> DeserializeWithSerdeContext<'de> for Box<T> {
    fn deserialize_with_ctx<D: serde::de::Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        todo!()
    }
}