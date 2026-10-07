use erased_serde::__private::serde::Serialize;
use serde::{Deserialize, Serializer};
use crate::dif::deserialize_with_serde_context::DeserializeWithSerdeContext;
use crate::dif::serde_context::SerdeContext;
use crate::preludes::derive::{CoreLibBaseTypeId, SerializeWithSerdeContext, ValueClassification};
use crate::values::core_values::boolean::Boolean;

impl Serialize for Boolean {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(self.0)
    }
}

impl SerializeWithSerdeContext for Boolean {
    fn serialize_with_ctx<S: Serializer>(&self, ctx: &SerdeContext<'_>, serializer: S) -> Result<S::Ok, S::Error> {
        ctx.serialize_core_value(
            self,
            CoreLibBaseTypeId::Boolean.into(),
            serializer,
            true,
        )
    }
}


impl<'de> DeserializeWithSerdeContext<'de> for Boolean {
    fn deserialize_with_ctx<D: serde::de::Deserializer<'de>>(
        _ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let b = bool::deserialize(deserializer)?;
        Ok(Boolean(b))
    }
}