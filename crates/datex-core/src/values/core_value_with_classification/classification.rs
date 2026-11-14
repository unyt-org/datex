use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::PointerAddress,
    traits::classification::Classification,
    types::entity_type::EntityType,
    values::{
        core_value_with_classification::CoreValueWithClassification,
        value::value_classification::{
            ValueClassification, ValueTag,
            unresolved_value_classification::UnresolvedValueClassification,
        },
    },
};

impl Classification for CoreValueWithClassification {
    fn entity_type(
        &self,
        _cache: &mut SharedReferencesCache,
    ) -> Option<EntityType> {
        self.classification.entity_type.clone()
    }
    fn entity_type_address(&self) -> Option<PointerAddress> {
        self.classification
            .entity_type
            .as_ref()
            .map(|entity_type| entity_type.pointer_address())
    }

    fn tag(&self) -> Option<ValueTag> {
        self.classification.tag.clone()
    }

    fn impls(&self) -> Vec<PointerAddress> {
        self.classification.impls.clone()
    }

    fn classification(
        &self,
        _cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        self.classification.clone()
    }

    fn unresolved_classification(&self) -> UnresolvedValueClassification {
        self.classification.clone().into()
    }
}
