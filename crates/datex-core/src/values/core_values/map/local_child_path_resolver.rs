use crate::traits::local_child_path_resolver::LocalChildPathResolver;
use crate::value_updates::errors::UpdateError;
use crate::values::core_values::map::Map;
use crate::values::value_container::value_key::ValueKey;
use crate::values::value_container::ValueContainer;

impl LocalChildPathResolver for Map {
    fn resolve_child(
        &mut self,
        key: &ValueKey,
    ) -> Result<&mut ValueContainer, UpdateError> {
        self.get_mut(key).map_err(UpdateError::access_error)
    }
}