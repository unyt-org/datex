use alloc::borrow::Cow;
use crate::shared_values::pointer_address::{LocalPointerAddress, PointerAddress};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PointerReferenceMutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnedPointer {
    /// Address of the owned pointer, must be a local pointer address
    address: LocalPointerAddress,
    // TODO: additional fields will probably be added later, e.g. previous owners
    // subscribers: Vec<(Endpoint, Permissions)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PointerReference {
    /// Address of the borrowed pointer, can be a local or remote pointer address
    address: PointerAddress,
    mutability: PointerReferenceMutability
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pointer {
    Owned(OwnedPointer),
    Reference(PointerReference),
}

impl Pointer {
    pub const NULL: Pointer = Pointer::Owned(OwnedPointer {
        address: LocalPointerAddress::NULL,
    });

    pub fn address(&self) -> Cow<PointerAddress> {
        match self {
            Pointer::Owned(owned) => Cow::Owned(PointerAddress::Local(owned.address.clone())),
            Pointer::Reference(borrowed) => Cow::Borrowed(&borrowed.address),
        }
    }

    /// Creates a new owned pointer with the given local pointer address
    pub(crate) fn new_owned(address: LocalPointerAddress) -> Self {
        Pointer::Owned(OwnedPointer { address })
    }
    
    /// Creates a new borrowed pointer with the given pointer address and mutability
    pub(crate) fn new_reference(address: PointerAddress, mutability: PointerReferenceMutability) -> Self {
        Pointer::Reference(PointerReference { address, mutability })
    }

    /// Gets an immutable reference to the pointer.
    pub(crate) fn get_reference(&self) -> PointerReference {
        let address = self.address();
        PointerReference {
            address: address.into_owned(),
            mutability: PointerReferenceMutability::Immutable,
        }
    }

    /// Gets a mutable reference to the pointer if possible, otherwise returns None.
    /// For owned pointers, a mutable reference can always be created.
    /// For borrowed pointers, a mutable reference can only be created if the original pointer is mutable.
    pub fn get_reference_mut(&self) -> Option<PointerReference> {
        let address = self.address();
        // mutable reference from reference is only possible if the original pointer is owned or if it's a mutable reference
        match self {
            Pointer::Owned(_) => Some(PointerReference {
                address: address.into_owned(),
                mutability: PointerReferenceMutability::Mutable,
            }),
            Pointer::Reference(reference) => {
                if reference.mutability == PointerReferenceMutability::Mutable {
                    Some(PointerReference {
                        address: address.into_owned(),
                        mutability: PointerReferenceMutability::Mutable,
                    })
                } else {
                    None
                }
            }
        }
    }
}