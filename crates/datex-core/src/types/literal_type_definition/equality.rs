use crate::{
    traits::structural_eq::StructuralEq,
    types::literal_type_definition::LiteralTypeDefinition,
};

impl StructuralEq for LiteralTypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}
