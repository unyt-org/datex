use crate::{
    traits::structural_eq::StructuralEq,
    types::type_definition::range::RangeTypeDefinition,
};

impl StructuralEq for RangeTypeDefinition {
    fn structural_eq(&self, other: &Self) -> bool {
        self.start.structural_eq(&other.start)
            && self.end.structural_eq(&other.end)
    }
}
