//! This module contains the definition of pointer addresses for shared containers.
use crate::prelude::*;

use crate::values::core_values::endpoint::Endpoint;
use binrw::{BinRead, BinWrite};
use core::{fmt::Display, result::Result};
use serde::{Deserialize, Serialize};

#[derive(BinWrite, BinRead, Debug, Clone, PartialEq, Eq, Hash)]
#[brw(little)]
pub struct SelfOwnedPointerAddress(pub [u8; 5]);

#[derive(
    BinWrite, BinRead, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash,
)]
#[brw(little)]
pub struct RemotePointerAddress(pub [u8; 26]);

impl RemotePointerAddress {
    pub fn for_endpoint(
        endpoint: &Endpoint,
        self_owned_pointer_address: &SelfOwnedPointerAddress,
    ) -> Self {
        let endpoint_slice = endpoint.to_slice();
        let mut address = [0u8; 26];
        address[..endpoint_slice.len()].copy_from_slice(&endpoint_slice);
        address[endpoint_slice.len()
            ..endpoint_slice.len() + self_owned_pointer_address.0.len()]
            .copy_from_slice(&self_owned_pointer_address.0);
        RemotePointerAddress(address)
    }

    pub fn to_address_string(&self) -> String {
        hex::encode(self.0)
    }

    /// Returns the endpoint part of the remote pointer address
    pub fn endpoint(&self) -> Endpoint {
        let mut endpoint = [0u8; 21];
        endpoint.copy_from_slice(&self.0[0..21]);
        Endpoint::from_slice(endpoint).unwrap()
    }

    /// Normalizes the pointer address to a self-owned address if it is a
    /// remote address with the same endpoint as the provided local endpoint.
    pub fn normalize(self, local_endpoint: &Endpoint) -> PointerAddress {
        if &self.endpoint() == local_endpoint {
            let mut id = [0u8; 5];
            id.copy_from_slice(&self.0[21..26]);
            PointerAddress::SelfOwned(SelfOwnedPointerAddress::new(id))
        } else {
            PointerAddress::Remote(self)
        }
    }
}

impl Display for RemotePointerAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "${}", self.to_address_string())
    }
}

impl SelfOwnedPointerAddress {
    pub fn new(address: [u8; 5]) -> Self {
        SelfOwnedPointerAddress(address)
    }

    pub fn to_address_string(&self) -> String {
        hex::encode(self.0)
    }
}

impl TryFrom<String> for SelfOwnedPointerAddress {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let hex_str = if let Some(stripped) = s.strip_prefix('$') {
            stripped
        } else {
            &s
        };

        hex::decode(hex_str)
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

#[derive(BinWrite, BinRead, Debug, Clone, PartialEq, Eq, Hash)]
#[brw(little)]
pub enum PointerAddress {
    // pointer with the local endpoint as origin
    // the full pointer id consists of the local endpoint id + this local id
    #[brw(magic = 0u8)]
    SelfOwned(SelfOwnedPointerAddress),
    #[brw(magic = 1u8)]
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

    /// Normalizes the pointer address to a self-owned address if it is a
    /// remote address with the same endpoint as the provided local endpoint.
    pub fn normalize(self, local_endpoint: &Endpoint) -> Self {
        match self {
            PointerAddress::Remote(remote_address) => {
                remote_address.normalize(local_endpoint)
            }
            _ => self,
        }
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

impl From<RemotePointerAddress> for PointerAddress {
    fn from(remote: RemotePointerAddress) -> Self {
        PointerAddress::Remote(remote)
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
        let addr_str = self.to_address_string();
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
            PointerAddress::SelfOwned(local_address) => &local_address.0,
            PointerAddress::Remote(addr) => &addr.0,
        }
    }
}
