use crate::shared_values::pointer_address::{OwnedPointerAddress, PointerAddress, ReferencedPointerAddress};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PointerReferenceMutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnedPointer {
    /// Address of the owned pointer, must be a local pointer address
    address: OwnedPointerAddress,
    // TODO: additional fields will probably be added later, e.g. previous owners
    // subscribers: Vec<(Endpoint, Permissions)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PointerReference {
    /// Address of the borrowed pointer, can be a local or remote pointer address
    address: ReferencedPointerAddress,
    mutability: PointerReferenceMutability
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pointer {
    Owned(OwnedPointer),
    Reference(PointerReference),
}

impl Pointer {
    pub const NULL: Pointer = Pointer::Owned(OwnedPointer {
        address: OwnedPointerAddress::NULL,
    });

    pub fn address(&self) -> PointerAddress {
        match self {
            Pointer::Owned(owned) => PointerAddress::Owned(owned.address.clone()),
            Pointer::Reference(borrowed) => PointerAddress::Referenced(borrowed.address.clone()),
        }
    }

    pub fn reference_mutability(&self) -> Option<&PointerReferenceMutability> {
        match self {
            Pointer::Owned(_) => None,
            Pointer::Reference(reference) => Some(&reference.mutability),
        }
    }

    /// Creates a new owned pointer with the given local pointer address
    pub(crate) fn new_owned(address: OwnedPointerAddress) -> Self {
        Pointer::Owned(OwnedPointer { address })
    }
    
    /// Creates a new borrowed pointer with the given pointer address and mutability
    pub(crate) fn new_reference(address: ReferencedPointerAddress, mutability: PointerReferenceMutability) -> Self {
        Pointer::Reference(PointerReference { address, mutability })
    }
}