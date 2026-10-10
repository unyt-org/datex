use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::classification::Classification,
    values::{
        borrowed_value_container::BorrowedValueContainer,
        value::value_classification::ValueClassification,
        value_container::ValueContainer,
    },
};

impl<'a> Classification for BorrowedValueContainer<'a> {
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        match self {
            BorrowedValueContainer::Local(value) => {
                Classification::classification(value, cache)
            }
            BorrowedValueContainer::Shared(_shared) => {
                ValueClassification::new_unclassified()
            }
        }
    }
}
