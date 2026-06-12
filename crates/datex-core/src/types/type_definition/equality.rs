use crate::{
    traits::structural_eq::StructuralEq, types::type_definition::TypeDefinition,
};

impl StructuralEq for TypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypeDefinition::Literal(a), TypeDefinition::Literal(b)) => {
                a.structural_eq(b)
            }
            (TypeDefinition::Union(a), TypeDefinition::Union(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (item_a, item_b) in a.iter().zip(b.iter()) {
                    if !item_a.structural_eq(item_b) {
                        return false;
                    }
                }
                true
            }
            (TypeDefinition::List(a), TypeDefinition::List(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (item_a, item_b) in a.iter().zip(b.iter()) {
                    if !item_a.structural_eq(item_b) {
                        return false;
                    }
                }
                true
            }
            (TypeDefinition::Nested(a), TypeDefinition::Nested(b)) => {
                a.structural_eq(b)
            }
            (TypeDefinition::Shared(a), TypeDefinition::Shared(b)) => {
                a.structural_eq(b)
            }
            (TypeDefinition::Callable(a), TypeDefinition::Callable(b)) => {
                a.structural_eq(b)
            }
            (TypeDefinition::CoreType(c), TypeDefinition::CoreType(d)) => c == d,
            (TypeDefinition::Map(a), TypeDefinition::Map(b)) => {
                a.structural_eq(b)
            }
            (TypeDefinition::Range(a), TypeDefinition::Range(b)) => {
                a.structural_eq(b)
            }
            (
                TypeDefinition::Intersection(a),
                TypeDefinition::Intersection(b),
            ) => a.structural_eq(b),
            _ => false,
        }
    }
}
