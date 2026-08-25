use crate::{
    types::{
        traits::type_match::{TypeSatisfiesValueContainer, TypeSuperset},
        r#type::Type,
        type_definition::TypeDefinition,
    },
    values::value_container::ValueContainer,
};

impl TypeSuperset<Type> for TypeDefinition {
    fn is_superset_of(&self, other: &Type) -> bool {
        match self {
            TypeDefinition::Union(union) => union.is_superset_of(other),
            TypeDefinition::Box(nested) => nested.is_superset_of(other),
            // TODO
            // TypeDefinition::Intersection(intersection) => intersection.is_superset_of(other),
            _ => false,
        }
    }
}

impl TypeSuperset<TypeDefinition> for TypeDefinition {
    fn is_superset_of(&self, other: &TypeDefinition) -> bool {
        match (self, other) {
            // literal supersets, e.g. 10 >= 10
            (
                TypeDefinition::Literal(self_literal),
                TypeDefinition::Literal(other_literal),
            ) => self_literal.is_superset_of(other_literal),

            // union supersets, e.g. 1|2|3 >= 1|2
            (
                TypeDefinition::Union(self_union),
                TypeDefinition::Union(other_union),
            ) => self_union.is_superset_of(other_union),

            // core supersets, e.g. integer >= integer/u8
            (
                TypeDefinition::CoreType(self_core),
                TypeDefinition::CoreType(other_core),
            ) => self_core.is_superset_of(other_core),

            // union supersets, e.g. 1|2|3 >= 1|2
            (
                TypeDefinition::ImplType(self_impl),
                TypeDefinition::ImplType(other_impl),
            ) => self_impl.is_superset_of(other_impl),

            // union superset with any TypeDefinition, e.g. 1|2 >= 1
            (TypeDefinition::Union(self_union), other) => {
                self_union.is_superset_of(other)
            }

            // core superset with any TypeDefinition, e.g. integer >= 1
            (TypeDefinition::CoreType(self_core), other) => {
                self_core.is_superset_of(other)
            }

            // core is not a superset of literal, e.g. integer >= 10 --> false
            (TypeDefinition::Literal(_), TypeDefinition::CoreType(_)) => false,

            (self_, TypeDefinition::Union(other_union)) => other_union
                .iter()
                .all(|other_member| self_.is_superset_of(other_member)),

            // other cross-variant matching - todo
            (x, y) => unimplemented!(
                "is_superset_of not implemented for {x:?} >= {y:?}"
            ),
        }
    }
}

impl TypeSatisfiesValueContainer for TypeDefinition {
    fn satisfies_value_container(&self, value: &ValueContainer) -> bool {
        match self {
            TypeDefinition::Literal(definition) => {
                definition.satisfies_value_container(value)
            }
            TypeDefinition::Union(definitions) => definitions
                .iter()
                .any(|definition| definition.satisfies_value_container(value)),
            TypeDefinition::CoreType(definition) => definition.satisfies_value_container(value),
            _ => {
                unimplemented!(
                    "satisfies_value_container not implemented for {self:?} >= {value:?}"
                )
            }
        }
    }
}
