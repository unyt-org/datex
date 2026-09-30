use crate::{
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::classification::Classification,
    values::{
        core_values::native::NativeCoreValue,
        value::value_classification::ValueClassification,
    },
};
impl Classification for NativeCoreValue {
    fn classification(
        &self,
        cache: &mut SharedReferencesCache,
    ) -> ValueClassification {
        self.value.classification(cache)
    }
}
