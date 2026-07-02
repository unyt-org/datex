use crate::{
    traits::structural_eq::StructuralEq,
    values::core_values::endpoint::Endpoint,
};

impl StructuralEq for Endpoint {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
