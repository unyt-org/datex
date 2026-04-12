use binrw::{BinRead, BinWrite};
use num_enum::TryFromPrimitive;
use crate::shared_values::pointer_address::{
    OwnedPointerAddress, PointerAddress, ReferencedPointerAddress,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive, Serialize, Deserialize,BinRead, BinWrite)]
#[brw(repr(u8))]
#[repr(u8)]
pub enum PointerReferenceMutability {
    Immutable = 0,
    Mutable = 1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnedPointer {
    /// Address of the owned pointer, must be a local pointer address
    address: OwnedPointerAddress,
    // TODO #766: additional fields will probably be added later, e.g. previous owners
    // subscribers: Vec<(Endpoint, Permissions)>,
}

impl OwnedPointer {
    pub const NULL: OwnedPointer = OwnedPointer {
        address: OwnedPointerAddress::NULL,
    };

    pub fn new(address: OwnedPointerAddress) -> Self {
        OwnedPointer { address }
    }

    pub fn address(&self) -> &OwnedPointerAddress {
        &self.address
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferencedPointer {
    /// Address of the borrowed pointer, can be a local or remote pointer address
    address: ReferencedPointerAddress,
}

impl ReferencedPointer {
    pub fn new(address: ReferencedPointerAddress) -> Self {
        ReferencedPointer { address }
    }
    pub fn address(&self) -> &ReferencedPointerAddress {
        &self.address
    }
}
