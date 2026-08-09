use crate::{
    prelude::*,
    types::{
        shared_container_containing_entity_type::SharedContainerContainingEntityType,
        r#type::Type,
    },
};
use core::fmt::Display;

/// Represents a definition of an "entity" type,
/// which describes a nominal type identified by a unique pointer id.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EntityTypeDefinition {
    pub(crate) definition_type: Type,
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
    pub fn new(definition: Type, name: String) -> EntityTypeDefinition {
        EntityTypeDefinition {
            definition_type: definition,
            name,
            allowed_variants: Vec::new(),
        }
    }
}

impl EntityTypeDefinition {
    /// Get the inner [Type]
    pub fn definition_type(&self) -> &Type {
        &self.definition_type
    }

    /// Replace the inner [Type] with a new one and return the old one
    pub fn replace_definition_type(&mut self, new_definition: Type) {
        self.definition_type = new_definition;
    }
}
