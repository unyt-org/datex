use crate::{
    shared_values::shared_container::SharedContainerMutability,
    types::definition::TypeDefinition,
};

use crate::{
    shared_values::pointer::ReferenceMutability,
    values::core_values::r#type::{LocalMutability, LocalReferenceMutability},
};
use modular_bitfield::Specifier;
use num_enum::TryFromPrimitive;
use strum::Display;

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
            TypeDefinition::SharedReference(_) => {
                TypeInstructionCode::SHARED_TYPE_REFERENCE
            }
            TypeDefinition::Unit => todo!(),
            TypeDefinition::Unknown => todo!(),
            TypeDefinition::Never => todo!(),
            TypeDefinition::Structural(_) => {
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
        }
    }
}

#[derive(Clone, Debug, PartialEq, Display, Specifier)]
#[bits = 2]
pub enum TypeReferenceMutabilityCode {
    MutableReference,   // &mut / 'mut
    ImmutableReference, // & / '
    Value,              // default
}

impl From<&TypeReferenceMutabilityCode> for Option<ReferenceMutability> {
    fn from(value: &TypeReferenceMutabilityCode) -> Self {
        match value {
            TypeReferenceMutabilityCode::MutableReference => {
                Some(ReferenceMutability::Mutable)
            }
            TypeReferenceMutabilityCode::ImmutableReference => {
                Some(ReferenceMutability::Immutable)
            }
            TypeReferenceMutabilityCode::Value => None,
        }
    }
}

impl From<&Option<ReferenceMutability>> for TypeReferenceMutabilityCode {
    fn from(value: &Option<ReferenceMutability>) -> Self {
        match value {
            Some(ReferenceMutability::Mutable) => {
                TypeReferenceMutabilityCode::MutableReference
            }
            Some(ReferenceMutability::Immutable) => {
                TypeReferenceMutabilityCode::ImmutableReference
            }
            None => TypeReferenceMutabilityCode::Value,
        }
    }
}

impl From<&Option<LocalReferenceMutability>> for TypeReferenceMutabilityCode {
    fn from(value: &Option<LocalReferenceMutability>) -> Self {
        match value {
            Some(LocalReferenceMutability::Mutable) => {
                TypeReferenceMutabilityCode::MutableReference
            }
            Some(LocalReferenceMutability::Immutable) => {
                TypeReferenceMutabilityCode::ImmutableReference
            }
            None => TypeReferenceMutabilityCode::Value,
        }
    }
}

impl From<&TypeReferenceMutabilityCode> for Option<LocalReferenceMutability> {
    fn from(value: &TypeReferenceMutabilityCode) -> Self {
        match value {
            TypeReferenceMutabilityCode::MutableReference => {
                Some(LocalReferenceMutability::Mutable)
            }
            TypeReferenceMutabilityCode::ImmutableReference => {
                Some(LocalReferenceMutability::Immutable)
            }
            TypeReferenceMutabilityCode::Value => None,
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
