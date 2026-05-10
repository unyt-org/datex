use crate::{
    traits::structural_eq::StructuralEq,
    values::core_values::callable::{Callable, CallableSignature},
};

impl StructuralEq for Callable {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl StructuralEq for CallableSignature {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
