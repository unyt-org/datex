use crate::{
    shared_values::shared_containers::SharedContainerMutability,
    types::structural_type_definition::TypeDefinition,
};

use modular_bitfield::Specifier;
use num_enum::TryFromPrimitive;
use strum::Display;
use crate::shared_values::shared_containers::{ReferenceMutability, SharedContainerOwnership};
use crate::types::type_definition::{LocalMutability, LocalReferenceMutability};

#[allow(non_camel_case_types)]
#[derive(
    Debug,
    Eq,
    PartialEq,
    TryFromPrimitive,
    Copy,
    Clone,
    Display,
    num_enum::IntoPrimitive,
)]
#[repr(u8)]
pub enum TypeInstructionCode {
    SHARED_TYPE_REFERENCE,
    TYPE_WITH_IMPLS,

    TYPE_LIST,
    TYPE_RANGE,

    TYPE_LITERAL_INTEGER,
    TYPE_LITERAL_TEXT,
    TYPE_LITERAL_SHORT_TEXT,
}

impl From<&TypeDefinition> for TypeInstructionCode {
    fn from(value: &TypeDefinition) -> Self {
        match value {
            TypeDefinition::ImplType(_, _) => {
                TypeInstructionCode::TYPE_WITH_IMPLS
            }
            TypeDefinition::Shared(_) => {
                TypeInstructionCode::SHARED_TYPE_REFERENCE
            }
            TypeDefinition::Unit => todo!(),
            TypeDefinition::Unknown => todo!(),
            TypeDefinition::Never => todo!(),
            TypeDefinition::Literal(_) => {
                todo!()
            }
            TypeDefinition::Intersection(_) => {
                todo!()
            }
            TypeDefinition::Union(_) => todo!(),
            TypeDefinition::Callable { .. } => {
                todo!()
            }
            TypeDefinition::Collection(_) => {
                todo!()
            }
            TypeDefinition::Type(_) => unreachable!(), // TODO #668: nested types
            TypeDefinition::List(_) => todo!(),
            TypeDefinition::Map(_) => todo!(),
            TypeDefinition::Range(_) => todo!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Display, Specifier)]
#[bits = 2]
pub enum TypeOwnershipCode {
    MutableReference,   // &mut / 'mut
    ImmutableReference, // & / '
    Value,              // default
}

impl From<&TypeOwnershipCode> for SharedContainerOwnership {
    fn from(value: &TypeOwnershipCode) -> Self {
        match value {
            TypeOwnershipCode::MutableReference => {
                SharedContainerOwnership::Referenced(ReferenceMutability::Mutable)
            }
            TypeOwnershipCode::ImmutableReference => {
                SharedContainerOwnership::Referenced(ReferenceMutability::Immutable)
            }
            TypeOwnershipCode::Value => SharedContainerOwnership::Owned,
        }
    }
}

impl From<&SharedContainerOwnership> for TypeOwnershipCode {
    fn from(value: &SharedContainerOwnership) -> Self {
        match value {
            SharedContainerOwnership::Referenced(ReferenceMutability::Mutable) => {
                TypeOwnershipCode::MutableReference
            }
            SharedContainerOwnership::Referenced(ReferenceMutability::Immutable) => {
                TypeOwnershipCode::ImmutableReference
            }
            SharedContainerOwnership::Owned => TypeOwnershipCode::ImmutableReference,
        }
    }
}

impl From<&Option<LocalReferenceMutability>> for TypeOwnershipCode {
    fn from(value: &Option<LocalReferenceMutability>) -> Self {
        match value {
            Some(LocalReferenceMutability::Mutable) => {
                TypeOwnershipCode::MutableReference
            }
            Some(LocalReferenceMutability::Immutable) => {
                TypeOwnershipCode::ImmutableReference
            }
            None => TypeOwnershipCode::Value,
        }
    }
}

impl From<&TypeOwnershipCode> for Option<LocalReferenceMutability> {
    fn from(value: &TypeOwnershipCode) -> Self {
        match value {
            TypeOwnershipCode::MutableReference => {
                Some(LocalReferenceMutability::Mutable)
            }
            TypeOwnershipCode::ImmutableReference => {
                Some(LocalReferenceMutability::Immutable)
            }
            TypeOwnershipCode::Value => None,
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
