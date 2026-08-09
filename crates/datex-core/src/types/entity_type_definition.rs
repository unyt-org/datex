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
pub enum EntityTypeDefinition {
    Base {
        definition_type: Type,
        name: String,
    },
    Variant {
        definition_type: Type,
        base: SharedContainerContainingEntityType,
        variant_name: String,
    },
}

impl Display for EntityTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EntityTypeDefinition::Base { name, .. } => write!(f, "{}", name),
            EntityTypeDefinition::Variant {
                base, variant_name, ..
            } => write!(
                f,
                "{}/{}",
                base.with_collapsed_definition(|def| def.to_string()),
                variant_name
            ),
        }
    }
}

impl EntityTypeDefinition {
    pub fn new_base(definition: Type, name: String) -> EntityTypeDefinition {
        EntityTypeDefinition::Base {
            definition_type: definition,
            name,
        }
    }

    pub fn new_variant(
        definition: Type,
        base: SharedContainerContainingEntityType,
        variant_name: String,
    ) -> EntityTypeDefinition {
        EntityTypeDefinition::Variant {
            definition_type: definition,
            base,
            variant_name,
        }
    }

    /// Get the inner [Type]
    pub fn definition_type(&self) -> &Type {
        match self {
            EntityTypeDefinition::Base {
                definition_type: definition,
                ..
            } => definition,
            EntityTypeDefinition::Variant {
                definition_type: definition,
                ..
            } => definition,
        }
    }

    /// Replace the inner [Type] with a new one and return the old one
    pub fn replace_definition_type(&mut self, new_definition: Type) {
        match self {
            EntityTypeDefinition::Base {
                definition_type: definition,
                ..
            } => *definition = new_definition,
            EntityTypeDefinition::Variant {
                definition_type: definition,
                ..
            } => *definition = new_definition,
        }
    }

    /// Convert to the inner [Type]
    pub fn into_definition_type(self) -> Type {
        match self {
            EntityTypeDefinition::Base {
                definition_type: definition,
                ..
            } => definition,
            EntityTypeDefinition::Variant {
                definition_type: definition,
                ..
            } => definition,
        }
    }
}
