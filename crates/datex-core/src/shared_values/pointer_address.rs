use crate::prelude::*;

use crate::{
    global::protocol_structures::instruction_data::{
        RawRemotePointerAddress,
        RawSelfOwnedPointerAddress,
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
pub struct RemotePointerAddress(pub [u8; 26]);


impl RemotePointerAddress {
    pub fn for_endpoint(endpoint: &Endpoint, id: [u8; 5]) -> Self {
        let endpoint_slice = endpoint.to_slice();
        let mut address = [0u8; 26];
        address[..endpoint_slice.len()].copy_from_slice(&endpoint_slice);
        address[endpoint_slice.len()..endpoint_slice.len() + id.len()]
            .copy_from_slice(&id);
        RemotePointerAddress(address)
    }

    pub fn to_address_string(&self) -> String {
        hex::encode(self.0)
    }
}

impl Display for RemotePointerAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "${}", self.to_address_string())
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

impl TryFrom<String> for SelfOwnedPointerAddress {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        hex::decode(s)
            .map_err(|_| "Invalid hex string for SelfOwnedPointerAddress")
            .and_then(|bytes| {
                if bytes.len() == 5 {
                    let mut arr = [0u8; 5];
                    arr.copy_from_slice(&bytes);
                    Ok(SelfOwnedPointerAddress::new(arr))
                } else {
                    Err("SelfOwnedPointerAddress must be 5 bytes long")
                }
            })
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
    Remote(RemotePointerAddress),
}

impl PointerAddress {
    pub fn self_owned(address: [u8; 5]) -> Self {
        PointerAddress::SelfOwned(SelfOwnedPointerAddress::new(address))
    }

    pub fn remote(address: [u8; 26]) -> Self {
        PointerAddress::Remote(RemotePointerAddress(address))
    }

    pub fn remote_for_endpoint(endpoint: &Endpoint, id: [u8; 5]) -> Self {
        PointerAddress::Remote(RemotePointerAddress::for_endpoint(
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
                Ok(PointerAddress::Remote(RemotePointerAddress(arr)))
            }
            _ => Err("PointerAddress must be 5 or 26 bytes long"),
        }
    }
}

impl From<SelfOwnedPointerAddress> for PointerAddress {
    fn from(owned: SelfOwnedPointerAddress) -> Self {
        PointerAddress::SelfOwned(owned)
    }
}

impl From<RawSelfOwnedPointerAddress> for PointerAddress {
    fn from(raw: RawSelfOwnedPointerAddress) -> Self {
        PointerAddress::SelfOwned(SelfOwnedPointerAddress::new(raw.bytes))
    }
}


impl From<RawRemotePointerAddress> for PointerAddress {
    fn from(raw: RawRemotePointerAddress) -> Self {
        PointerAddress::Remote(RemotePointerAddress(raw.id))
    }
}

impl PointerAddress {
    pub fn to_address_string(&self) -> String {
        match self {
            PointerAddress::SelfOwned(local_address) => {
                local_address.to_address_string()
            }
            PointerAddress::Remote(address) => address.to_address_string(),
        }
    }
}

impl Display for PointerAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PointerAddress::SelfOwned(local_address) => {
                core::write!(f, "{}", local_address)
            }
            PointerAddress::Remote(address) => {
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
            PointerAddress::Remote(addr) => {
                &addr.0
            }
        }
    }
}
