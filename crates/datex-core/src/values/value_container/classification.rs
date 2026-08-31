use crate::preludes::derive::SharedReferencesCache;
use crate::traits::classification::Classification;
use crate::values::value::value_classification::ValueClassification;
use crate::values::value_container::ValueContainer;

impl Classification for ValueContainer {
    fn classification(&self, cache: &mut SharedReferencesCache) -> ValueClassification {
        match self {
            ValueContainer::Local(value) => Classification::classification(value, cache),
            ValueContainer::Shared(shared) => ValueClassification::None,
        }
    }
}