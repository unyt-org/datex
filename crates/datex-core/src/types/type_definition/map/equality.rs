use crate::{
    traits::structural_eq::StructuralEq,
    types::type_definition::map::MapTypeDefinition,
};

impl StructuralEq for MapTypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for ((key_a, value_a), (key_b, value_b)) in
            self.iter().zip(other.iter())
        {
            if !key_a.structural_eq(key_b) || !value_a.structural_eq(value_b) {
                return false;
            }
        }
        true
    }
}
