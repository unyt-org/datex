use serde::{Serialize, Serializer};
use crate::dif::deserialize_with_serde_context::{DeserializeWithSerdeContext, DeserializeWithSerdeContextDyn};
use crate::dif::serde_context::SerdeContext;
use crate::dif::serialize_with_serde_context::SerializeWithSerdeContextDyn;
use crate::preludes::derive::{SerializeWithSerdeContext};

impl<T: SerializeWithSerdeContextDyn> SerializeWithSerdeContext for Option<T> {
    fn serialize_with_ctx<S: Serializer>(&self, ctx: &SerdeContext<'_>, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Some(value) => {
                value.with_ctx(ctx).serialize(serializer)
            },
            None => serializer.serialize_none(),
        }
    }
}


impl<'de, T: DeserializeWithSerdeContextDyn> DeserializeWithSerdeContext<'de> for Option<T> {
    fn deserialize_with_ctx<D: serde::de::Deserializer<'de>>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        todo!()
    }
}