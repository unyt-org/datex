use crate::shared_values::errors::AccessError;
use crate::traits::local_child_path_resolver::LocalChildPathResolver;
use crate::value_updates::errors::UpdateError;
use crate::values::core_values::list::List;
use crate::values::value_container::value_key::ValueKey;
use crate::values::value_container::ValueContainer;

impl LocalChildPathResolver for List {
    fn resolve_child(
        &mut self,
        key: &ValueKey,
    ) -> Result<&mut ValueContainer, UpdateError> {
        if let ValueKey::Index(index) = key {
            self.try_get_mut(*index)
                .map_err(UpdateError::access_error)
        }
        else {
            Err(UpdateError::access_error(AccessError::InvalidIndexKey))
        }
    }
}