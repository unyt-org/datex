use crate::{
    traits::structural_eq::StructuralEq, values::core_values::list::List,
};
use core::hash::{Hash, Hasher};

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl Eq for List {}

impl Hash for List {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.items.hash(state);
    }
}

impl StructuralEq for List {
    fn structural_eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (a, b) in self.items.iter().zip(other.items.iter()) {
            if !a.structural_eq(b) {
                return false;
            }
        }
        true
    }
}
