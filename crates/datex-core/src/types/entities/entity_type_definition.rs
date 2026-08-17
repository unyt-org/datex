use super::entity_impls::EntityImpl;
use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    prelude::*,
    types::type_definition::TypeDefinition,
    values::core_values::callable::Callable,
};
use core::fmt::Display;

/// Represents a definition of an "entity" type,
/// which describes a nominal type identified by a unique pointer id.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EntityTypeDefinition {
    pub(crate) definition: TypeDefinition,
    pub(crate) name: String,
    pub(crate) allowed_variants: Vec<String>,
    pub(crate) impls: Vec<EntityImpl>,
}

impl Display for EntityTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl EntityTypeDefinition {
    pub fn new(
        definition: TypeDefinition,
        name: String,
    ) -> EntityTypeDefinition {
        EntityTypeDefinition {
            definition,
            name,
            allowed_variants: Vec::new(),
            impls: Vec::new(),
        }
    }

    pub fn new_with_impls(
        definition: TypeDefinition,
        name: String,
        impls: Vec<EntityImpl>,
    ) -> EntityTypeDefinition {
        EntityTypeDefinition {
            definition,
            name,
            allowed_variants: Vec::new(),
            impls,
        }
    }

    pub fn impls(&self) -> &[EntityImpl] {
        &self.impls
    }

    pub fn try_get_property(&self, property_name: &str) -> Option<&Callable> {
        self.impls
            .iter()
            .filter_map(|impl_ty| impl_ty.try_get_property(property_name))
            .next()
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
