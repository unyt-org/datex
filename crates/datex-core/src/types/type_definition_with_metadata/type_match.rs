use crate::{
    types::{
        type_definition_with_metadata::TypeDefinitionWithMetadata,
        type_match::TypeMatch,
    },
    values::value_container::ValueContainer,
};

impl TypeMatch for TypeDefinitionWithMetadata {
    fn matches(&self, definition: &Self) -> bool {
        if !self.metadata.matches(&definition.metadata) {
            return false;
        }
        // FIXME
        false
    }

    fn matched_by_value(&self, value: &ValueContainer) -> bool {
        self.definition.matched_by_value(value)
    }
}
