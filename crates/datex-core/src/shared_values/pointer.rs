use alloc::borrow::Cow;
use crate::shared_values::pointer_address::{LocalPointerAddress, PointerAddress};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BorrowedPointerMutability {
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
pub struct BorrowedPointer {
    /// Address of the borrowed pointer, can be a local or remote pointer address
    address: PointerAddress,
    mutability: BorrowedPointerMutability
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pointer {
    Owned(OwnedPointer),
    Borrowed(BorrowedPointer),
}

impl Pointer {
    pub const NULL: Pointer = Pointer::Owned(OwnedPointer {
        address: LocalPointerAddress::NULL,
    });

    pub fn address(&self) -> Cow<PointerAddress> {
        match self {
            Pointer::Owned(owned) => Cow::Owned(PointerAddress::Local(owned.address.clone())),
            Pointer::Borrowed(borrowed) => Cow::Borrowed(&borrowed.address),
        }
    }

    /// Creates a new owned pointer with the given local pointer address
    pub(crate) fn new_owned(address: LocalPointerAddress) -> Self {
        Pointer::Owned(OwnedPointer { address })
    }
    
    /// Creates a new borrowed pointer with the given pointer address and mutability
    pub(crate) fn new_borrowed(address: PointerAddress, mutability: BorrowedPointerMutability) -> Self {
        Pointer::Borrowed(BorrowedPointer { address, mutability })
    }
}