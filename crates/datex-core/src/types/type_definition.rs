use serde::Serialize;
use crate::serde::Deserialize;
use crate::shared_values::shared_containers::{SharedContainerMutability, SharedContainerOwnership};
use crate::types::literal_type_definition::LiteralTypeDefinition;
use crate::types::r#type::Type;
use crate::types::structural_type_definition::StructuralTypeDefinition;

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

impl Default for TypeMetadata {
    fn default() -> Self {
        TypeMetadata::Local {
            mutability: LocalMutability::Immutable,
            reference_mutability: None,
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeDefinition {
    pub structural_definition: StructuralTypeDefinition,
    pub metadata: TypeMetadata,
}

impl From<TypeDefinition> for Type {
    fn from(x: TypeDefinition) -> Self {
        Type::Alias(x)
    }
}