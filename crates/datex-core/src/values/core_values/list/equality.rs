use crate::{
    traits::structural_eq::StructuralEq, values::core_values::list::List,
};

impl StructuralEq for List {
    fn structural_eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            if !a.structural_eq(b) {
                return false;
            }
        }
        true
    }
}
