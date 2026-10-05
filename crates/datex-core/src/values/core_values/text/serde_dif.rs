use serde::{Deserialize, Serializer};
use crate::dif::deserialize_with_serde_context::DeserializeWithSerdeContext;
use crate::dif::serde_context::SerdeContext;
use crate::preludes::derive::SerializeWithSerdeContext;
use crate::values::core_values::text::Text;
use crate::prelude::*;

impl SerializeWithSerdeContext for Text {
    fn serialize_with_ctx<S: Serializer>(&self, _ctx: &SerdeContext<'_>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> DeserializeWithSerdeContext<'de> for Text {
    fn deserialize_with_ctx<D: serde::de::Deserializer<'de>>(
        _ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Text(s))
    }
}