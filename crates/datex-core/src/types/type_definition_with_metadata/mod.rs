use crate::{prelude::*, types::type_definition::TypeDefinition};
use core::{fmt::Display, hash::Hash};

pub mod metadata;
pub mod serde_dif;
pub use metadata::*;
pub mod type_match;
#[derive(Debug, Eq, Clone)]
pub struct TypeDefinitionWithMetadata {
    pub definition: TypeDefinition,
    pub metadata: TypeMetadata,
    reference_name: Option<String>,
}
impl TypeDefinitionWithMetadata {
    pub const fn unit() -> Self {
        TypeDefinitionWithMetadata {
            definition: TypeDefinition::UNIT,
            metadata: TypeMetadata::default(),
            reference_name: None,
        }
    }
    pub const fn null() -> Self {
        TypeDefinitionWithMetadata {
            definition: TypeDefinition::NULL,
            metadata: TypeMetadata::default(),
            reference_name: None,
        }
    }
}

impl TypeDefinitionWithMetadata {
    pub fn new(definition: TypeDefinition, metadata: TypeMetadata) -> Self {
        Self {
            definition,
            metadata,
            reference_name: None,
        }
    }
    /// Internal function to set the reference name of the type definition. This is used when creating a new type definition with a name.
    pub(crate) fn set_reference_name(&mut self, name: String) {
        self.reference_name = Some(name);
    }
    pub fn reference_name(&self) -> Option<&str> {
        self.reference_name.as_deref()
    }
}

impl PartialEq for TypeDefinitionWithMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.definition == other.definition && self.metadata == other.metadata
    }
}

impl Hash for TypeDefinitionWithMetadata {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.definition.hash(state);
        self.metadata.hash(state);
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
