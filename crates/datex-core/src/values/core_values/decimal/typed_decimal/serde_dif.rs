use erased_serde::__private::serde::{Serialize, Serializer};
use serde::Deserialize;
use crate::dif::deserialize_with_serde_context::DeserializeWithSerdeContext;
use crate::dif::serde_context::SerdeContext;
use crate::dif::serialize_with_serde_context::SerializeWithSerdeContext;
use crate::libs::core::type_id::CoreLibVariantTypeId;
use crate::preludes::derive::{CoreLibBaseTypeId, ValueClassification};
use crate::values::core_values::decimal::typed_decimal::TypedDecimal;


impl SerializeWithSerdeContext for TypedDecimal {
    fn serialize_with_ctx<S: Serializer>(&self, ctx: &SerdeContext<'_>, serializer: S) -> Result<S::Ok, S::Error> {
        ctx.serialize_core_value(
            self,
            CoreLibVariantTypeId::Decimal(self.variant()).into(),
            &ValueClassification::default(),
            serializer,
            false,
        )
    }
}

impl<'de> DeserializeWithSerdeContext<'de> for TypedDecimal {
    fn deserialize_with_ctx<D: serde::de::Deserializer<'de>>(
        _ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        todo!()
    }
}