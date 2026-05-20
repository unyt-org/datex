use crate::{
    types::{type_definition::TypeDefinition, type_match::TypeMatch},
    values::value_container::ValueContainer,
};

impl TypeMatch for TypeDefinition {
    fn matches(&self, definition: &Self) -> bool {
        match (self, definition) {
            (TypeDefinition::Literal(lhs), TypeDefinition::Literal(rhs)) => {
                lhs.matches(rhs)
            }
            (TypeDefinition::Union(lhs), TypeDefinition::Union(rhs)) => {
                if lhs.len() != rhs.len() {
                    return false;
                }
                lhs.iter()
                    .zip(rhs.iter())
                    .all(|(lhs_def, rhs_def)| lhs_def.matches(rhs_def))
            }
            _ => unimplemented!(),
        }
    }

    fn matched_by_value(&self, value: &ValueContainer) -> bool {
        match self {
            TypeDefinition::Literal(definition) => {
                definition.matched_by_value(value)
            }
            TypeDefinition::Union(definitions) => definitions
                .iter()
                .any(|definition| definition.matched_by_value(value)),
            _ => unimplemented!(),
        }
    }
}
