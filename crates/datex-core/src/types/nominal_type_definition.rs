use crate::types::shared_container_containing_type::SharedContainerContainingType;
use crate::types::type_definition::TypeDefinition;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum NominalTypeDefinition {
    Base {
        definition: TypeDefinition,
        name: String
    },
    Variant {
        definition: TypeDefinition,
        base: SharedContainerContainingType,
        variant_name: String,
    }
}

impl NominalTypeDefinition {
    pub fn new_base(definition: TypeDefinition, name: String) -> NominalTypeDefinition {
        NominalTypeDefinition::Base { definition, name }
    }
    
    pub fn new_variant(definition: TypeDefinition, base: SharedContainerContainingType, variant_name: String) -> NominalTypeDefinition {
        NominalTypeDefinition::Variant { definition, base, variant_name }
    }
    
    /// Get the inner [TypeDefinition]
    pub fn definition(&self) -> &TypeDefinition {
        match self {
            NominalTypeDefinition::Base { definition, .. } => definition,
            NominalTypeDefinition::Variant { definition, .. } => definition,
        }
    }

    /// Convert to the inner [TypeDefinition]
    pub fn into_definition(self) -> TypeDefinition {
        match self {
            NominalTypeDefinition::Base { definition, .. } => definition,
            NominalTypeDefinition::Variant { definition, .. } => definition,
        }
    }
}