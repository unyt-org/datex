use crate::{
    traits::structural_eq::StructuralEq, values::core_values::integer::Integer,
};

impl StructuralEq for Integer {
    fn structural_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
