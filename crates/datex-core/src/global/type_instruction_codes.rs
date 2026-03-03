use crate::{
    shared_values::shared_container::SharedContainerMutability,
    types::definition::TypeDefinition,
};

use crate::{
    shared_values::pointer::PointerReferenceMutability,
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
    TYPE_UNIT,
    TYPE_UNKNOWN,
    TYPE_NEVER,
    TYPE_STRUCTURAL,
    TYPE_INTERSECTION,
    TYPE_UNION,
    TYPE_FUNCTION,
    TYPE_COLLECTION,
    TYPE_TYPE,

    TYPE_LIST,
    TYPE_RANGE,

    TYPE_LITERAL_INTEGER,
    TYPE_LITERAL_TEXT,
    TYPE_LITERAL_SHORT_TEXT,
    TYPE_STRUCT,

    // TODO #427: Do we need std_type for optimization purpose?
    // Rename to CORE_ and implement if required
    // but TYPE TYPE_TEXT is already two bytes which is not a great benefit over the three
    // bytes for the internal pointer address + GETREF (4 vs 2 bytes)
    STD_TYPE_TEXT,
    STD_TYPE_INT,
    STD_TYPE_FLOAT,
    STD_TYPE_BOOLEAN,
    STD_TYPE_NULL,
    STD_TYPE_VOID,
    STD_TYPE_BUFFER,
    STD_TYPE_CODE_BLOCK,
    STD_TYPE_QUANTITY,
    STD_TYPE_TIME,
    STD_TYPE_URL,

    STD_TYPE_ARRAY,
    STD_TYPE_OBJECT,
    STD_TYPE_SET,
    STD_TYPE_MAP,
    STD_TYPE_TUPLE,

    STD_TYPE_FUNCTION,
    STD_TYPE_STREAM,
    STD_TYPE_ANY,
    STD_TYPE_ASSERTION,
    STD_TYPE_TASK,
    STD_TYPE_ITERATOR,
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
            TypeDefinition::Unit => TypeInstructionCode::TYPE_UNIT,
            TypeDefinition::Unknown => TypeInstructionCode::TYPE_UNKNOWN,
            TypeDefinition::Never => TypeInstructionCode::TYPE_NEVER,
            TypeDefinition::Structural(_) => {
                TypeInstructionCode::TYPE_STRUCTURAL
            }
            TypeDefinition::Intersection(_) => {
                TypeInstructionCode::TYPE_INTERSECTION
            }
            TypeDefinition::Union(_) => TypeInstructionCode::TYPE_UNION,
            TypeDefinition::Callable { .. } => {
                TypeInstructionCode::TYPE_FUNCTION
            }
            TypeDefinition::Collection(_) => {
                TypeInstructionCode::TYPE_COLLECTION
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

impl From<&TypeReferenceMutabilityCode> for Option<PointerReferenceMutability> {
    fn from(value: &TypeReferenceMutabilityCode) -> Self {
        match value {
            TypeReferenceMutabilityCode::MutableReference => {
                Some(PointerReferenceMutability::Mutable)
            }
            TypeReferenceMutabilityCode::ImmutableReference => {
                Some(PointerReferenceMutability::Immutable)
            }
            TypeReferenceMutabilityCode::Value => None,
        }
    }
}

impl From<&Option<PointerReferenceMutability>> for TypeReferenceMutabilityCode {
    fn from(value: &Option<PointerReferenceMutability>) -> Self {
        match value {
            Some(PointerReferenceMutability::Mutable) => {
                TypeReferenceMutabilityCode::MutableReference
            }
            Some(PointerReferenceMutability::Immutable) => {
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
