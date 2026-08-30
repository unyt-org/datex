use crate::values::core_values::native::DatexNative;
use crate::values::value::value_classification::ValueClassification;

/// Marker trait indicating that this value does not have an entity type.
/// This guarantees that [DatexNative::entity_type] will always return None for this value.
pub trait DatexNativeStructural: DatexNative {

    /// Returns the DATEX [ValueClassification] of the native value.
    /// This only checks for the presence of tag, since entity type resolution requires a cache.
    fn classification_without_cache(&self) -> ValueClassification {
        match self.tag() {
            Some(tag) => ValueClassification::Tag(tag),
            None => ValueClassification::None,
        }
        // TODO: impl types?
    }
}