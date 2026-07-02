use crate::{
    traits::structural_eq::StructuralEq,
    types::type_definition::callable::CallableTypeDefinition,
    values::core_values::callable::Callable,
};

impl StructuralEq for Callable {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
impl StructuralEq for CallableTypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
