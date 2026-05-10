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
            (TypeDefinition::Core(c), TypeDefinition::Core(d)) => c == d,
            (TypeDefinition::Map(a), TypeDefinition::Map(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for ((key_a, value_a), (key_b, value_b)) in
                    a.iter().zip(b.iter())
                {
                    if !key_a.structural_eq(key_b)
                        || !value_a.structural_eq(value_b)
                    {
                        return false;
                    }
                }
                true
            }
            (
                TypeDefinition::Range((a_start, a_end)),
                TypeDefinition::Range((b_start, b_end)),
            ) => a_start.structural_eq(b_start) && a_end.structural_eq(b_end),
            (
                TypeDefinition::Intersection(a),
                TypeDefinition::Intersection(b),
            ) => {
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
            _ => false,
        }
    }
}
