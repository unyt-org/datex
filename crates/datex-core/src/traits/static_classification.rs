use crate::traits::classification::Classification;

pub trait StaticClassification: Classification {
    /// Returns true if the value has a classification (either an entity type or a tag).
    /// Should only return false if [Classification::classification] returns [ValueClassification::none()].
    fn has_classification() -> bool {
        false
    }
}
