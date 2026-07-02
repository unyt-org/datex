use crate::{traits::structural_eq::StructuralEq, types::r#type::Type};

impl StructuralEq for Type {
    // FIXME is this what we want?
    fn structural_eq(&self, other: &Self) -> bool {
        self.with_collapsed_definition_with_metadata(|own| {
            other.with_collapsed_definition_with_metadata(|other| {
                own.definition.structural_eq(&other.definition)
            })
        })
    }
}
