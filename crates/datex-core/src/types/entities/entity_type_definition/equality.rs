use crate::types::entities::entity_type_definition::EntityTypeDefinition;

impl PartialEq for EntityTypeDefinition {
    fn eq(&self, other: &Self) -> bool {
        // this ignores deserialize_from_dif
        self.name == other.name
            && self.definition == other.definition
            && self.allowed_variants == other.allowed_variants
            && self.impls == other.impls
    }
}
