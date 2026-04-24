use crate::{
    traits::structural_eq::StructuralEq,
    values::core_values::callable::Callable,
};

impl StructuralEq for Callable {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
