use crate::{
    prelude::*,
    types::{
        shared_container_containing_entity_type::SharedContainerContainingEntityType,
        r#type::Type,
    },
};
use core::fmt::Display;
use crate::types::type_definition::TypeDefinition;

/// Represents a definition of an "entity" type,
/// which describes a nominal type identified by a unique pointer id.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EntityTypeDefinition {
    pub(crate) definition: TypeDefinition,
    pub(crate) name: String,
    pub(crate) allowed_variants: Vec<String>,
    // TODO: impls
}

impl Display for EntityTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl EntityTypeDefinition {
    pub fn new(definition: TypeDefinition, name: String) -> EntityTypeDefinition {
        EntityTypeDefinition {
            definition,
            name,
            allowed_variants: Vec::new(),
        }
    }
}

impl EntityTypeDefinition {
    /// Get the inner [TypeDefinition]
    pub fn definition(&self) -> &TypeDefinition {
        &self.definition
    }

    /// Replace the inner [TypeDefinition] with a new one and return the old one
    pub fn replace_definition(&mut self, new_definition: TypeDefinition) {
        self.definition = new_definition;
    }
}
