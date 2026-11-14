use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::PointerAddress,
    traits::classification::Classification,
    types::entity_type::EntityType,
    values::{
        core_values::native::NativeCoreValue,
        value::value_classification::ValueTag,
    },
};

impl Classification for NativeCoreValue {
    fn entity_type(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> Option<EntityType> {
        self.value.entity_type(cache)
    }

    fn entity_type_address(&self) -> Option<PointerAddress> {
        self.value.entity_type_address()
    }

    fn tag(&self) -> Option<ValueTag> {
        self.value.tag()
    }

    fn impls(&self) -> Vec<PointerAddress> {
        self.value.impls()
    }
}
