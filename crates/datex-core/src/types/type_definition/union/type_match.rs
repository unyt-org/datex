use crate::types::{
    r#type::Type,
    type_definition::{TypeDefinition, union::UnionTypeDefinition},
    traits::type_match::TypeSuperset,
};

impl TypeSuperset<Type> for UnionTypeDefinition {
    fn is_superset_of(&self, other: &Type) -> bool {
        // any type in self must be a superset of other
        self.iter().any(|self_type| self_type.is_superset_of(other))
    }
}

impl TypeSuperset<TypeDefinition> for UnionTypeDefinition {
    fn is_superset_of(&self, other: &TypeDefinition) -> bool {
        // any type in self must be a superset of other
        self.iter().any(|self_type| self_type.is_superset_of(other))
    }
}

impl TypeSuperset<UnionTypeDefinition> for UnionTypeDefinition {
    fn is_superset_of(&self, other: &UnionTypeDefinition) -> bool {
        // all types in other must be a subset of self
        // e.g. 1 | text <= integer | text | decimal -> true
        other
            .iter()
            .all(|other_type| self.is_superset_of(other_type))
    }
}
