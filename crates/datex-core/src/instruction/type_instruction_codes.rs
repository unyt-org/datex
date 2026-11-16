use crate::{
    shared_values::SharedContainerMutability,
    types::type_definition_with_metadata::LocalOwnership,
};

use crate::{
    shared_values::{ReferenceMutability, SharedContainerOwnership},
    types::type_definition_with_metadata::{
        LocalMutability, LocalReferenceMutability,
    },
};
use modular_bitfield::Specifier;
use strum::Display;

#[derive(Clone, Debug, PartialEq, Display, Specifier)]
#[bits = 2]
pub enum TypeOwnershipCode {
    MutableReference,   // &mut / 'mut
    ImmutableReference, // & / '
    Owned,              // default
}

impl From<&TypeOwnershipCode> for SharedContainerOwnership {
    fn from(value: &TypeOwnershipCode) -> Self {
        match value {
            TypeOwnershipCode::MutableReference => {
                SharedContainerOwnership::Referenced(
                    ReferenceMutability::Mutable,
                )
            }
            TypeOwnershipCode::ImmutableReference => {
                SharedContainerOwnership::Referenced(
                    ReferenceMutability::Immutable,
                )
            }
            TypeOwnershipCode::Owned => SharedContainerOwnership::Owned,
        }
    }
}

impl From<&SharedContainerOwnership> for TypeOwnershipCode {
    fn from(value: &SharedContainerOwnership) -> Self {
        match value {
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Mutable,
            ) => TypeOwnershipCode::MutableReference,
            SharedContainerOwnership::Referenced(
                ReferenceMutability::Immutable,
            ) => TypeOwnershipCode::ImmutableReference,
            SharedContainerOwnership::Owned => {
                TypeOwnershipCode::ImmutableReference
            }
        }
    }
}

impl From<&LocalOwnership> for TypeOwnershipCode {
    fn from(value: &LocalOwnership) -> Self {
        match value {
            LocalOwnership::Owned => TypeOwnershipCode::Owned,
            LocalOwnership::Referenced(LocalReferenceMutability::Mutable) => {
                TypeOwnershipCode::MutableReference
            }
            LocalOwnership::Referenced(LocalReferenceMutability::Immutable) => {
                TypeOwnershipCode::ImmutableReference
            }
        }
    }
}

impl From<&TypeOwnershipCode> for LocalOwnership {
    fn from(value: &TypeOwnershipCode) -> Self {
        match value {
            TypeOwnershipCode::Owned => LocalOwnership::Owned,
            TypeOwnershipCode::MutableReference => {
                LocalOwnership::Referenced(LocalReferenceMutability::Mutable)
            }
            TypeOwnershipCode::ImmutableReference => {
                LocalOwnership::Referenced(LocalReferenceMutability::Immutable)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Display, Specifier)]
#[bits = 1]
pub enum TypeMutabilityCode {
    Mutable,   // mut / shared mut
    Immutable, // default or shared
}

impl From<&TypeMutabilityCode> for SharedContainerMutability {
    fn from(value: &TypeMutabilityCode) -> Self {
        match value {
            TypeMutabilityCode::Mutable => SharedContainerMutability::Mutable,
            TypeMutabilityCode::Immutable => {
                SharedContainerMutability::Immutable
            }
        }
    }
}

impl From<&SharedContainerMutability> for TypeMutabilityCode {
    fn from(value: &SharedContainerMutability) -> Self {
        match value {
            SharedContainerMutability::Mutable => TypeMutabilityCode::Mutable,
            SharedContainerMutability::Immutable => {
                TypeMutabilityCode::Immutable
            }
        }
    }
}

impl From<&TypeMutabilityCode> for LocalMutability {
    fn from(value: &TypeMutabilityCode) -> Self {
        match value {
            TypeMutabilityCode::Mutable => LocalMutability::Mutable,
            TypeMutabilityCode::Immutable => LocalMutability::Immutable,
        }
    }
}

impl From<&LocalMutability> for TypeMutabilityCode {
    fn from(value: &LocalMutability) -> Self {
        match value {
            LocalMutability::Mutable => TypeMutabilityCode::Mutable,
            LocalMutability::Immutable => TypeMutabilityCode::Immutable,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Display, Specifier)]
#[bits = 1]
pub enum TypeLocalOrShared {
    Local,  // default
    Shared, // shared
}
