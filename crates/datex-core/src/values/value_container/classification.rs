use crate::{
    preludes::derive::SharedReferencesCache,
    traits::classification::Classification,
    values::{
        value::value_classification::ValueClassification,
        value_container::ValueContainer,
    },
};

impl Classification for ValueContainer {
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        match self {
            ValueContainer::Local(value) => {
                Classification::classification(value, cache)
            }
            ValueContainer::Shared(_shared) => ValueClassification::None,
        }
    }
}
