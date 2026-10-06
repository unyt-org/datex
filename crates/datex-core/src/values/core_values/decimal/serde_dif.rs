use serde::{Deserialize, Serializer};
use crate::dif::deserialize_with_serde_context::DeserializeWithSerdeContext;
use crate::dif::serde_context::SerdeContext;
use crate::preludes::derive::{CoreLibBaseTypeId, SerializeWithSerdeContext, ValueClassification};
use crate::prelude::*;
use crate::values::core_values::decimal::Decimal;

impl SerializeWithSerdeContext for Decimal {
    fn serialize_with_ctx<S: Serializer>(&self, ctx: &SerdeContext<'_>, serializer: S) -> Result<S::Ok, S::Error> {
        ctx.serialize_core_value(
            self, 
            CoreLibBaseTypeId::Decimal.into(),
            &ValueClassification::default(),
            serializer,
            false,
        )
    }
}

impl<'de> DeserializeWithSerdeContext<'de> for Decimal {
    fn deserialize_with_ctx<D: serde::de::Deserializer<'de>>(
        _ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Decimal::try_from_string(&s).map_err(serde::de::Error::custom)?)
    }
}