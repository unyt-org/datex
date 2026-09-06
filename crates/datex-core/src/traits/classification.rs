use crate::{
    preludes::derive::SharedReferencesCache,
    types::entity_type::EntityType,
    values::value::value_classification::{ValueClassification, ValueTag},
};

pub trait Classification {
    /// Returns the DATEX [EntityType] of the native value if it has an entity type.
    /// For structural types, this will return None.
    /// The default implementation returns None, indicating that the value does not have an entity type.
    fn entity_type(
        &self,
        _cache: &mut SharedReferencesCache,
    ) -> Option<EntityType> {
        None
    }

    /// Returns a [ValueTag] if the value has a tag.
    /// The default implementation returns None, indicating that the value does not have a tag.
    fn tag(&self) -> Option<ValueTag> {
        None
    }

    /// Returns the DATEX [ValueClassification] of the native value.
    /// This tries to resolve the entity type and tag, assuming at most one of them is present.
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        if let Some(entity_type) = self.entity_type(cache) {
            ValueClassification::Entity(entity_type)
        } else if let Some(tag) = self.tag() {
            ValueClassification::Tag(tag)
        } else {
            ValueClassification::None
        }
        // TODO: impl types?
    }
}
