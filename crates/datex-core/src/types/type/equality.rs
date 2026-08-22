use crate::{traits::structural_eq::StructuralEq, types::r#type::Type};

impl StructuralEq for Type {
    // FIXME is this what we want?
    fn structural_eq(&self, other: &Self) -> bool {
        self.as_definition_with_metadata(|own| {
            other.as_definition_with_metadata(|other| {
                own.definition.structural_eq(&other.definition)
            })
        })
    }
}
