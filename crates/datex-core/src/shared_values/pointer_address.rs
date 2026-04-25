use crate::prelude::*;

use crate::{
    global::protocol_structures::instruction_data::{
        RawBuiltinPointerAddress, RawLocalPointerAddress, RawPointerAddress,
        RawRemotePointerAddress,
    },
    values::core_values::endpoint::Endpoint,
};
use core::{fmt::Display, result::Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelfOwnedPointerAddress {
    pub(crate) address: [u8; 5],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExternalPointerAddress {
    // pointer with a remote endpoint as origin, contains the full pointers address
    Remote([u8; 26]),
    // globally unique builtin pointer, e.g. for #core, #std
    Builtin([u8; 3]), // TODO #312 shrink down to 2 bytes?
}

impl ExternalPointerAddress {
    pub fn remote_for_endpoint(endpoint: &Endpoint, id: [u8; 5]) -> Self {
        let endpoint_slice = endpoint.to_slice();
        let mut address = [0u8; 26];
        address[..endpoint_slice.len()].copy_from_slice(&endpoint_slice);
        address[endpoint_slice.len()..endpoint_slice.len() + id.len()]
            .copy_from_slice(&id);
        ExternalPointerAddress::Remote(address)
    }

    pub fn to_address_string(&self) -> String {
        match self {
            ExternalPointerAddress::Remote(bytes) => hex::encode(bytes),
            ExternalPointerAddress::Builtin(bytes) => hex::encode(bytes),
        }
    }
}

impl Display for ExternalPointerAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExternalPointerAddress::Remote(_) => {
                core::write!(f, "${}", self.to_address_string())
            }
            ExternalPointerAddress::Builtin(_) => {
                core::write!(f, "{}", self.to_address_string())
            }
        }
    }
}

impl SelfOwnedPointerAddress {
    pub fn new(address: [u8; 5]) -> Self {
        SelfOwnedPointerAddress { address }
    }

    pub fn to_address_string(&self) -> String {
        hex::encode(self.address)
    }
}

impl Display for SelfOwnedPointerAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "${}", self.to_address_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PointerAddress {
    // pointer with the local endpoint as origin
    // the full pointer id consists of the local endpoint id + this local id
    SelfOwned(SelfOwnedPointerAddress),
    // pointer with a remote endpoint as origin, contains the full pointers address
    External(ExternalPointerAddress),
}

impl PointerAddress {
    pub fn self_owned(address: [u8; 5]) -> Self {
        PointerAddress::SelfOwned(SelfOwnedPointerAddress::new(address))
    }

    pub fn builtin(address: [u8; 3]) -> Self {
        PointerAddress::External(ExternalPointerAddress::Builtin(address))
    }

    pub fn remote(address: [u8; 26]) -> Self {
        PointerAddress::External(ExternalPointerAddress::Remote(address))
    }

    pub fn remote_for_endpoint(endpoint: &Endpoint, id: [u8; 5]) -> Self {
        PointerAddress::External(ExternalPointerAddress::remote_for_endpoint(
            endpoint, id,
        ))
    }
}

impl TryFrom<String> for PointerAddress {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        PointerAddress::try_from(s.as_str())
    }
}
impl TryFrom<&str> for PointerAddress {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let hex_str = if let Some(stripped) = s.strip_prefix('$') {
            stripped
        } else {
            s
        };
        let bytes = hex::decode(hex_str).map_err(|_| "Invalid hex string")?;
        match bytes.len() {
            5 => {
                let mut arr = [0u8; 5];
                arr.copy_from_slice(&bytes);
                Ok(PointerAddress::SelfOwned(SelfOwnedPointerAddress::new(arr)))
            }
            26 => {
                let mut arr = [0u8; 26];
                arr.copy_from_slice(&bytes);
                Ok(PointerAddress::External(ExternalPointerAddress::Remote(
                    arr,
                )))
            }
            3 => {
                let mut arr = [0u8; 3];
                arr.copy_from_slice(&bytes);
                Ok(PointerAddress::External(ExternalPointerAddress::Builtin(
                    arr,
                )))
            }
            _ => Err("PointerAddress must be 5, 26 or 3 bytes long"),
        }
    }
}

impl From<RawPointerAddress> for PointerAddress {
    fn from(raw: RawPointerAddress) -> Self {
        match raw {
            RawPointerAddress::Remote(remote) => PointerAddress::External(
                ExternalPointerAddress::Remote(remote.id),
            ),
            RawPointerAddress::Internal(internal) => PointerAddress::External(
                ExternalPointerAddress::Builtin(internal.id),
            ),
            RawPointerAddress::Local(local) => {
                PointerAddress::SelfOwned(SelfOwnedPointerAddress {
                    address: local.bytes,
                })
            }
        }
    }
}

impl From<SelfOwnedPointerAddress> for PointerAddress {
    fn from(owned: SelfOwnedPointerAddress) -> Self {
        PointerAddress::SelfOwned(owned)
    }
}

impl From<ExternalPointerAddress> for PointerAddress {
    fn from(referenced: ExternalPointerAddress) -> Self {
        PointerAddress::External(referenced)
    }
}

impl From<&RawLocalPointerAddress> for PointerAddress {
    fn from(raw: &RawLocalPointerAddress) -> Self {
        PointerAddress::SelfOwned(SelfOwnedPointerAddress::new(raw.bytes))
    }
}

impl From<&RawBuiltinPointerAddress> for PointerAddress {
    fn from(raw: &RawBuiltinPointerAddress) -> Self {
        PointerAddress::External(ExternalPointerAddress::Builtin(raw.id))
    }
}

impl From<&RawRemotePointerAddress> for PointerAddress {
    fn from(raw: &RawRemotePointerAddress) -> Self {
        PointerAddress::External(ExternalPointerAddress::Remote(raw.id))
    }
}

impl From<&RawPointerAddress> for PointerAddress {
    fn from(raw: &RawPointerAddress) -> Self {
        match raw {
            RawPointerAddress::Local(bytes) => PointerAddress::SelfOwned(
                SelfOwnedPointerAddress::new(bytes.bytes),
            ),
            RawPointerAddress::Internal(bytes) => PointerAddress::External(
                ExternalPointerAddress::Builtin(bytes.id),
            ),
            RawPointerAddress::Remote(bytes) => PointerAddress::External(
                ExternalPointerAddress::Remote(bytes.id),
            ),
        }
    }
}

impl PointerAddress {
    pub fn to_address_string(&self) -> String {
        match self {
            PointerAddress::SelfOwned(local_address) => {
                local_address.to_address_string()
            }
            PointerAddress::External(address) => address.to_address_string(),
        }
    }
}

impl Display for PointerAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PointerAddress::SelfOwned(local_address) => {
                core::write!(f, "{}", local_address)
            }
            PointerAddress::External(address) => {
                core::write!(f, "{}", address)
            }
        }
    }
}
impl Serialize for PointerAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let addr_str = self.to_string();
        serializer.serialize_str(&addr_str)
    }
}
impl<'de> Deserialize<'de> for PointerAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PointerAddress::try_from(s.as_str()).map_err(|e| {
            serde::de::Error::custom(format!(
                "Failed to parse PointerAddress: {}",
                e
            ))
        })
    }
}

impl PointerAddress {
    pub fn bytes(&self) -> &[u8] {
        match self {
            PointerAddress::SelfOwned(local_address) => &local_address.address,
            PointerAddress::External(ExternalPointerAddress::Remote(bytes)) => {
                bytes
            }
            PointerAddress::External(ExternalPointerAddress::Builtin(
                bytes,
            )) => bytes,
        }
    }

    pub fn internal_bytes(&self) -> Option<&[u8; 3]> {
        if let PointerAddress::External(ExternalPointerAddress::Builtin(
            bytes,
        )) = self
        {
            Some(bytes)
        } else {
            None
        }
    }
}
