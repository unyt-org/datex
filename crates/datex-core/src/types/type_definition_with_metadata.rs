use crate::shared_values::shared_containers::ReferenceMutability;
use core::fmt::Display;

use crate::{
    prelude::*,
    serde::Deserialize,
    shared_values::shared_containers::{
        SharedContainerMutability, SharedContainerOwnership,
    },
    types::{type_definition::TypeDefinition, type_match::TypeMatch},
    values::value_container::ValueContainer,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocalReferenceMutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocalMutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Combination of &/&mut, '/'mut shared and mut prefixes
pub enum TypeMetadata {
    /// Local types can be mut or not, and can optionally be a reference type with an additional reference mutability (e.g. &mut User)
    Local {
        mutability: LocalMutability,
        reference_mutability: Option<LocalReferenceMutability>,
    },
    /// Shared types are always (shared or shared mut) and can optionally be a non-owned, reference type
    /// with an additional reference mutability (e.g. 'mut shared mut User)
    Shared {
        mutability: SharedContainerMutability,
        ownership: SharedContainerOwnership,
    },
}

impl Display for TypeMetadata {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TypeMetadata::Local {
                mutability,
                reference_mutability,
            } => {
                let mutability_str = match mutability {
                    LocalMutability::Mutable => "mut ",
                    LocalMutability::Immutable => "",
                };
                let reference_str = match reference_mutability {
                    Some(LocalReferenceMutability::Mutable) => "&mut ",
                    Some(LocalReferenceMutability::Immutable) => "&",
                    None => "",
                };
                write!(f, "{}{}", reference_str, mutability_str)
            }
            TypeMetadata::Shared {
                mutability,
                ownership,
            } => {
                write!(f, "{}", ownership)?;
                if let SharedContainerOwnership::Referenced(
                    ReferenceMutability::Mutable,
                ) = ownership
                {
                    write!(f, " ")?
                };
                write!(f, "{}", mutability)
            }
        }
    }
}

impl TypeMetadata {
    /// Ownership type for a shared container
    pub fn shared_container_ownership(
        &self,
    ) -> Option<&SharedContainerOwnership> {
        match self {
            TypeMetadata::Local { .. } => None,
            TypeMetadata::Shared { ownership, .. } => Some(ownership),
        }
    }

    /// Mutability for a shared type (e.g. shared mut X / shared X), if applicable
    pub fn shared_mutability(&self) -> Option<SharedContainerMutability> {
        match self {
            TypeMetadata::Local { .. } => None,
            TypeMetadata::Shared { mutability, .. } => Some(mutability.clone()),
        }
    }

    /// Mutability for a reference to a local type (e.g. &mut X), if applicable
    pub fn local_reference_mutability(
        &self,
    ) -> Option<LocalReferenceMutability> {
        match self {
            TypeMetadata::Local {
                reference_mutability: local_reference_mutability,
                ..
            } => local_reference_mutability.clone(),
            TypeMetadata::Shared { .. } => None,
        }
    }

    /// Whether this type is a shared type (e.g. shared X, shared mut X, &shared X, &mut shared X)
    pub fn is_shared_type(&self) -> bool {
        match self {
            TypeMetadata::Shared { .. } => true,
            TypeMetadata::Local { .. } => false,
        }
    }
}

impl TypeMatch for TypeMetadata {
    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (
                TypeMetadata::Local {
                    mutability: mutability1,
                    reference_mutability: reference_mutability1,
                },
                TypeMetadata::Local {
                    mutability: mutability2,
                    reference_mutability: reference_mutability2,
                },
            ) => {
                mutability1 == mutability2
                    && reference_mutability1 == reference_mutability2
            }
            (
                TypeMetadata::Shared {
                    mutability: mutability1,
                    ownership: ownership1,
                },
                TypeMetadata::Shared {
                    mutability: mutability2,
                    ownership: ownership2,
                },
            ) => mutability1 == mutability2 && ownership1 == ownership2,
            _ => false,
        }
    }

    fn matched_by_value(&self, _value: &ValueContainer) -> bool {
        unimplemented!()
    }
}

impl Default for TypeMetadata {
    fn default() -> Self {
        TypeMetadata::Local {
            mutability: LocalMutability::Immutable,
            reference_mutability: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeDefinitionWithMetadata {
    pub definition: TypeDefinition,
    pub metadata: TypeMetadata,
}

impl TypeMatch for TypeDefinitionWithMetadata {
    fn matches(&self, definition: &Self) -> bool {
        if !self.metadata.matches(&definition.metadata) {
            return false;
        }
        // FIXME
        false
    }

    fn matched_by_value(&self, _value: &ValueContainer) -> bool {
        todo!()
    }
}

impl Display for TypeDefinitionWithMetadata {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let metadata_str = self.metadata.to_string();
        if !metadata_str.is_empty() {
            write!(f, "{} ", metadata_str)?;
        }
        write!(f, "{}", self.definition)
    }
}
