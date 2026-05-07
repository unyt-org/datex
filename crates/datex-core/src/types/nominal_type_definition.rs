use crate::{
    prelude::*,
    types::{
        shared_container_containing_nominal_type::SharedContainerContainingNominalType,
        r#type::Type,
    },
};
use core::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum NominalTypeDefinition {
    Base {
        definition_type: Type,
        name: String,
    },
    Variant {
        definition_type: Type,
        base: SharedContainerContainingNominalType,
        variant_name: String,
    },
}

impl Display for NominalTypeDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NominalTypeDefinition::Base { name, .. } => write!(f, "{}", name),
            NominalTypeDefinition::Variant {
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

impl NominalTypeDefinition {
    pub fn new_base(definition: Type, name: String) -> NominalTypeDefinition {
        NominalTypeDefinition::Base {
            definition_type: definition,
            name,
        }
    }

    pub fn new_variant(
        definition: Type,
        base: SharedContainerContainingNominalType,
        variant_name: String,
    ) -> NominalTypeDefinition {
        NominalTypeDefinition::Variant {
            definition_type: definition,
            base,
            variant_name,
        }
    }

    /// Get the inner [Type]
    pub fn definition_type(&self) -> &Type {
        match self {
            NominalTypeDefinition::Base {
                definition_type: definition,
                ..
            } => definition,
            NominalTypeDefinition::Variant {
                definition_type: definition,
                ..
            } => definition,
        }
    }

    /// Convert to the inner [Type]
    pub fn into_definition_type(self) -> Type {
        match self {
            NominalTypeDefinition::Base {
                definition_type: definition,
                ..
            } => definition,
            NominalTypeDefinition::Variant {
                definition_type: definition,
                ..
            } => definition,
        }
    }
}
