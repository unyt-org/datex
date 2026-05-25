use crate::{
    traits::structural_eq::StructuralEq,
    types::type_definition::intersection::IntersectionTypeDefinition,
};

impl StructuralEq for IntersectionTypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for item in self.iter() {
            if !other
                .iter()
                .any(|other_item| item.structural_eq(other_item))
            {
                return false;
            }
        }
        true
    }
}
