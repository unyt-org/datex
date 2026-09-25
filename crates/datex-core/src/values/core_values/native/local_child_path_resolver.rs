use crate::{
    traits::local_child_path_resolver::LocalChildPathResolver,
    value_updates::errors::UpdateError,
    values::{
        core_values::native::NativeCoreValue,
        value::Value,
        value_container::{ValueContainer, value_key::ValueKey},
    },
};

impl LocalChildPathResolver for NativeCoreValue {
    fn resolve_child(
        &mut self,
        key: &ValueKey,
    ) -> Result<&mut ValueContainer, UpdateError> {
        self.value.resolve_child(key)
    }

    fn resolve_value_for_path(
        &mut self,
        first: &ValueKey,
        remaining_path: &[ValueKey],
    ) -> Result<&mut Value, UpdateError> {
        self.value.resolve_value_for_path(first, remaining_path)
    }
}
