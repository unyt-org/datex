use crate::{
    traits::structural_eq::StructuralEq, values::core_values::boolean::Boolean,
};

impl StructuralEq for Boolean {
    fn structural_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
