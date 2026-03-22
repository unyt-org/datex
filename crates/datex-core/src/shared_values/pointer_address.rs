use crate::{
    prelude::*,
};

use core::{fmt::Display, result::Result};
use serde::{Deserialize, Serialize};
use crate::global::protocol_structures::instruction_data::{RawInternalPointerAddress, RawLocalPointerAddress, RawPointerAddress, RawRemotePointerAddress};
use crate::values::core_values::endpoint::Endpoint;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnedPointerAddress {
    pub(crate) address: [u8; 5],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReferencedPointerAddress {
    // pointer with a remote endpoint as origin, contains the full pointers address
    Remote([u8; 26]),
    // globally unique internal pointer, e.g. for #core, #std
    Internal([u8; 3]), // TODO #312 shrink down to 2 bytes?
}

impl ReferencedPointerAddress {
    pub fn remote_for_endpoint(endpoint: &Endpoint, id: [u8; 5]) -> Self {
        let endpoint_slice = endpoint.to_slice();
        let mut address = [0u8; 26];
        address[..endpoint_slice.len()].copy_from_slice(&endpoint_slice);
        address[endpoint_slice.len()..endpoint_slice.len() + id.len()].copy_from_slice(&id);
        ReferencedPointerAddress::Remote(address)
    }
}

impl OwnedPointerAddress {
    pub fn new(address: [u8; 5]) -> Self {
        OwnedPointerAddress { address }
    }

    pub const NULL: OwnedPointerAddress =
        OwnedPointerAddress { address: [0u8; 5] };
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PointerAddress {
    // pointer with the local endpoint as origin
    // the full pointer id consists of the local endpoint id + this local id
    Owned(OwnedPointerAddress),
    // pointer with a remote endpoint as origin, contains the full pointers address
    Referenced(ReferencedPointerAddress),
}

impl PointerAddress {
    pub const NULL: PointerAddress =
        PointerAddress::Owned(OwnedPointerAddress::NULL);

    pub fn owned(address: [u8; 5]) -> Self {
        PointerAddress::Owned(OwnedPointerAddress::new(address))
    }

    pub fn internal(address: [u8; 3]) -> Self {
        PointerAddress::Referenced(ReferencedPointerAddress::Internal(address))
    }

    pub fn remote(address: [u8; 26]) -> Self {
        PointerAddress::Referenced(ReferencedPointerAddress::Remote(address))
    }
    
    pub fn remote_for_endpoint(endpoint: &Endpoint, id: [u8; 5]) -> Self {
        PointerAddress::Referenced(ReferencedPointerAddress::remote_for_endpoint(endpoint, id))
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
                Ok(PointerAddress::Owned(OwnedPointerAddress::new(arr)))
            }
            26 => {
                let mut arr = [0u8; 26];
                arr.copy_from_slice(&bytes);
                Ok(PointerAddress::Referenced(
                    ReferencedPointerAddress::Remote(arr),
                ))
            }
            3 => {
                let mut arr = [0u8; 3];
                arr.copy_from_slice(&bytes);
                Ok(PointerAddress::Referenced(
                    ReferencedPointerAddress::Internal(arr),
                ))
            }
            _ => Err("PointerAddress must be 5, 26 or 3 bytes long"),
        }
    }
}

impl From<RawPointerAddress> for PointerAddress {
    fn from(raw: RawPointerAddress) -> Self {
        match raw {
            RawPointerAddress::Remote(remote) => PointerAddress::Referenced(
                ReferencedPointerAddress::Remote(remote.id),
            ),
            RawPointerAddress::Internal(internal) => PointerAddress::Referenced(
                ReferencedPointerAddress::Internal(internal.id),
            ),
            RawPointerAddress::Local(local) => PointerAddress::Owned(
                OwnedPointerAddress { address: local.id },
            ),
        }
    }
}

impl From<OwnedPointerAddress> for PointerAddress {
    fn from(owned: OwnedPointerAddress) -> Self {
        PointerAddress::Owned(owned)
    }
}

impl From<ReferencedPointerAddress> for PointerAddress {
    fn from(referenced: ReferencedPointerAddress) -> Self {
        PointerAddress::Referenced(referenced)
    }
}

impl From<&RawLocalPointerAddress> for PointerAddress {
    fn from(raw: &RawLocalPointerAddress) -> Self {
        PointerAddress::Owned(OwnedPointerAddress::new(raw.id))
    }
}

impl From<&RawInternalPointerAddress> for PointerAddress {
    fn from(raw: &RawInternalPointerAddress) -> Self {
        PointerAddress::Referenced(ReferencedPointerAddress::Internal(raw.id))
    }
}

impl From<&RawRemotePointerAddress> for PointerAddress {
    fn from(raw: &RawRemotePointerAddress) -> Self {
        PointerAddress::Referenced(ReferencedPointerAddress::Remote(raw.id))
    }
}

impl From<&RawPointerAddress> for PointerAddress {
    fn from(raw: &RawPointerAddress) -> Self {
        match raw {
            RawPointerAddress::Local(bytes) => {
                PointerAddress::Owned(OwnedPointerAddress::new(bytes.id))
            }
            RawPointerAddress::Internal(bytes) => PointerAddress::Referenced(
                ReferencedPointerAddress::Internal(bytes.id),
            ),
            RawPointerAddress::Remote(bytes) => PointerAddress::Referenced(
                ReferencedPointerAddress::Remote(bytes.id),
            ),
        }
    }
}

impl PointerAddress {
    pub fn to_address_string(&self) -> String {
        match self {
            PointerAddress::Owned(local_address) => {
                hex::encode(local_address.address)
            }
            PointerAddress::Referenced(ReferencedPointerAddress::Remote(
                bytes,
            )) => hex::encode(bytes),
            PointerAddress::Referenced(ReferencedPointerAddress::Internal(
                bytes,
            )) => hex::encode(bytes),
        }
    }
}

impl Display for PointerAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "$")?;
        core::write!(f, "{}", self.to_address_string())
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
            PointerAddress::Owned(local_address) => &local_address.address,
            PointerAddress::Referenced(ReferencedPointerAddress::Remote(
                bytes,
            )) => bytes,
            PointerAddress::Referenced(ReferencedPointerAddress::Internal(
                bytes,
            )) => bytes,
        }
    }

    pub fn internal_bytes(&self) -> Option<&[u8; 3]> {
        if let PointerAddress::Referenced(ReferencedPointerAddress::Internal(
            bytes,
        )) = self
        {
            Some(bytes)
        } else {
            None
        }
    }
}
