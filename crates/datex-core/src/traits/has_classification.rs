use crate::traits::classification::Classification;

pub trait HasClassification: Classification {
    /// Returns true if the value has a classification (either an entity type or a tag).
    /// Should only return false if [Classification::classification] returns [ValueClassification::None].
    fn has_classification() -> bool {
        false
    }
}