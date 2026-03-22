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


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pointer {
    Owned(OwnedPointer),
    Referenced(ReferencedPointer),
}

// REQUEST
// perform_move 10 -> (REQUEST @original move $1->$4)
// reponse
// ...

impl Pointer {
    pub const NULL: Pointer = Pointer::Owned(OwnedPointer {
        address: OwnedPointerAddress::NULL,
    });

    pub fn address(&self) -> PointerAddress {
        match self {
            Pointer::Owned(owned) => {
                PointerAddress::Owned(owned.address.clone())
            }
            Pointer::Referenced(borrowed) => {
                PointerAddress::Referenced(borrowed.address.clone())
            }
        }
    }

    pub fn is_owned(&self) -> bool {
        matches!(self, Pointer::Owned(_))
    }

    pub fn is_referenced(&self) -> bool {
        matches!(self, Pointer::Referenced(_))
    }

    /// Creates a new owned pointer with the given local pointer address
    pub(crate) fn new_owned(address: OwnedPointerAddress) -> Self {
        Pointer::Owned(OwnedPointer { address })
    }

    /// Creates a new borrowed pointer with the given pointer address and mutability
    pub(crate) fn new_reference(
        address: ReferencedPointerAddress,
    ) -> Self {
        Pointer::Referenced(ReferencedPointer {
            address,
        })
    }
}
