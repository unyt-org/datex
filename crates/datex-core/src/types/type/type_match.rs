use crate::{
    types::{r#type::Type, type_match::TypeSatisfiesValueContainer},
    values::value_container::ValueContainer,
};
use crate::types::type_definition::TypeDefinition;
use crate::types::type_match::TypeSuperset;

impl TypeSuperset<Type> for Type {
    /// Checks if this type is a superset of the other type, i.e. if all values that match the other type also match this type.
    ///
    /// Examples:
    /// integer >= 1 -> true
    /// 1 >= integer -> false
    /// integer >= integer -> true
    /// integer | text >= 1 -> true
    fn is_superset_of(&self, other: &Type) -> bool {
        match self {
            Type::Alias(self_definition) => self_definition.is_superset_of(other),
            Type::Nominal(self_nominal_definition) => {
                todo!()
            }
        }
    }
}

impl TypeSuperset<TypeDefinition> for Type {
    fn is_superset_of(&self, other: &TypeDefinition) -> bool {
        match self {
            Type::Alias(self_definition) => self_definition.is_superset_of(other),
            Type::Nominal(self_nominal_definition) => {
                todo!()
            }
        }
    }
}

impl TypeSatisfiesValueContainer for Type {
    fn satisfies_value_container(&self, value: &ValueContainer) -> bool {
        match self {
            Type::Alias(definition) => definition.satisfies_value_container(value),
            Type::Nominal(definition) => definition.satisfies_value_container(value),
        }
    }
}
