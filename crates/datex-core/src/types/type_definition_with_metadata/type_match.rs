use crate::{
    types::{
        type_definition_with_metadata::TypeDefinitionWithMetadata,
        type_match::TypeSatisfiesValueContainer,
    },
    values::value_container::ValueContainer,
};
use crate::types::r#type::Type;
use crate::types::type_definition::TypeDefinition;
use crate::types::type_definition_with_metadata::TypeMetadata;
use crate::types::type_match::TypeSuperset;

impl TypeSuperset<Type> for TypeDefinitionWithMetadata {
    fn is_superset_of(&self, other: &Type) -> bool {
        match other {
            Type::Alias(other_definition) => self.is_superset_of(other_definition),
            Type::Nominal(_) => {
                // direct nominal type, has implicit default metadata
                if TypeMetadata::default().is_superset_of(&self.metadata) {
                    self.definition.is_superset_of(other)
                } else {
                    false
                }
            }
        }
    }
}

impl TypeSuperset<TypeDefinitionWithMetadata> for TypeDefinitionWithMetadata {
    fn is_superset_of(&self, other: &TypeDefinitionWithMetadata) -> bool {
        if !self.metadata.is_superset_of(&other.metadata) {
            return false;
        }
        self.definition.is_superset_of(&other.definition)
    }
}

impl TypeSuperset<TypeDefinition> for TypeDefinitionWithMetadata {
    fn is_superset_of(&self, other: &TypeDefinition) -> bool {
        // has implicit default metadata
        if !TypeMetadata::default().is_superset_of(&self.metadata) {
            return false;
        }
        self.definition.is_superset_of(other)
    }
}

impl TypeSatisfiesValueContainer for TypeDefinitionWithMetadata {
    fn satisfies_value_container(&self, value: &ValueContainer) -> bool {
        self.definition.satisfies_value_container(value)
    }
}
