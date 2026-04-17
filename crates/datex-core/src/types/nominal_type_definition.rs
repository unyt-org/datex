use crate::{
    prelude::*,
    types::{
        shared_container_containing_type::SharedContainerContainingType,
        type_definition::TypeDefinitionWithMetadata,
    },
};
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum NominalTypeDefinition {
    Base {
        definition: TypeDefinitionWithMetadata,
        name: String,
    },
    Variant {
        definition: TypeDefinitionWithMetadata,
        base: SharedContainerContainingType,
        variant_name: String,
    },
}

impl NominalTypeDefinition {
    pub fn new_base(
        definition: TypeDefinitionWithMetadata,
        name: String,
    ) -> NominalTypeDefinition {
        NominalTypeDefinition::Base { definition, name }
    }

    pub fn new_variant(
        definition: TypeDefinitionWithMetadata,
        base: SharedContainerContainingType,
        variant_name: String,
    ) -> NominalTypeDefinition {
        NominalTypeDefinition::Variant {
            definition,
            base,
            variant_name,
        }
    }

    /// Get the inner [TypeDefinition]
    pub fn definition(&self) -> &TypeDefinitionWithMetadata {
        match self {
            NominalTypeDefinition::Base { definition, .. } => definition,
            NominalTypeDefinition::Variant { definition, .. } => definition,
        }
    }

    /// Convert to the inner [TypeDefinition]
    pub fn into_definition(self) -> TypeDefinitionWithMetadata {
        match self {
            NominalTypeDefinition::Base { definition, .. } => definition,
            NominalTypeDefinition::Variant { definition, .. } => definition,
        }
    }
}
