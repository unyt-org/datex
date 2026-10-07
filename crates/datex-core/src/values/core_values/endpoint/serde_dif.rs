use crate::{
    dif::serde_context::SerdeContext, prelude::*,
    values::core_values::endpoint::Endpoint,
};
use alloc::string::String;
use core::fmt;
use erased_serde::__private::serde::Serializer;
use serde::{
    Deserialize, Serialize,
    de::{Error, Visitor},
};
use crate::dif::deserialize_serde_context::impl_serde_with_context;
use crate::dif::deserialize_with_serde_context::DeserializeWithSerdeContext;
use crate::dif::serialize_with_serde_context::SerializeWithSerdeContext;
use crate::preludes::derive::{CoreLibBaseTypeId, ValueClassification};
use crate::values::core_value::CoreValue;

impl Serialize for Endpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer
            .serialize_str(&self.to_string())
            .map_err(serde::ser::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Endpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Endpoint::from_string(&s).map_err(serde::de::Error::custom)
    }
}

/*
TODO: handle
   ctx.serialize_core_value(
            self,
            CoreLibBaseTypeId::Endpoint.into(),
            &ValueClassification::default(),
            serializer,
            false,
        )
in macro
 */

impl SerializeWithSerdeContext for Endpoint {
    fn serialize_with_ctx<S: Serializer>(
        &self,
        ctx: &SerdeContext<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ctx.serialize_core_value(
            self,
            CoreLibBaseTypeId::Endpoint.into(),
            serializer,
            false,
        )
    }
}

impl<'de> DeserializeWithSerdeContext<'de> for Endpoint {
    fn deserialize_with_ctx<D>(
        ctx: &SerdeContext<'_>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match ctx.deserialize_core_value(deserializer)? {
            CoreValue::Endpoint(endpoint) => Ok(endpoint),
            _ => Err(D::Error::custom("Expected CoreValue::Endpoint")),
        }
    }
}