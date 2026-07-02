use crate::{
    traits::structural_eq::StructuralEq, values::core_values::text::Text,
};

impl StructuralEq for Text {
    fn structural_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
