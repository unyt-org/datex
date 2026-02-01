use core::prelude::rust_2024::*;

use serde::Serialize;

use crate::{
    compat::{string::String, string::ToString},
    utils::{uuid::UUID},
};
use core::fmt::Display;

#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm_runtime", tsify(type = "string"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComInterfaceSocketUUID(pub(crate) UUID);
impl Display for ComInterfaceSocketUUID {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::write!(f, "socket::{}", self.0)
    }
}

impl TryFrom<String> for ComInterfaceSocketUUID {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.strip_prefix("socket::").ok_or(())?;
        Ok(ComInterfaceSocketUUID(UUID::from_string(value.to_string())))
    }
}

impl Serialize for ComInterfaceSocketUUID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> serde::Deserialize<'de> for ComInterfaceSocketUUID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ComInterfaceSocketUUID::try_from(s).map_err(|_| {
            serde::de::Error::custom("Invalid ComInterfaceSocketUUID")
        })
    }
}