use crate::{
    dif::serde_context::SerdeContext, prelude::*,
    values::core_values::endpoint::Endpoint,
};
use alloc::string::String;
use core::fmt;
use serde::{
    Deserialize, Serialize,
    de::{Error, Visitor},
};

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

impl<'a> Deserialize<'a> for Endpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Endpoint::from_string(&s).map_err(serde::de::Error::custom)
    }
}

impl<'de, 'ctx> Visitor<'de> for SerdeContext<'ctx, Endpoint> {
    type Value = Endpoint;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an endpoint string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Endpoint, E>
    where
        E: Error,
    {
        Endpoint::from_string(value).map_err(E::custom)
    }

    fn visit_string<E>(self, value: String) -> Result<Endpoint, E>
    where
        E: Error,
    {
        Endpoint::from_string(&value).map_err(E::custom)
    }
}
