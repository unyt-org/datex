use core::hash::Hash;

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
impl Hash for EntityTypeDefinition {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.definition.hash(state);
        self.allowed_variants.hash(state);
        self.impls.hash(state);
    }
}
