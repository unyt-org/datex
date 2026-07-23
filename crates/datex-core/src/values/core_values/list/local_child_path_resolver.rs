use crate::{
    shared_values::errors::AccessError,
    traits::local_child_path_resolver::LocalChildPathResolver,
    value_updates::errors::UpdateError,
    values::{
        core_values::list::List,
        value_container::{ValueContainer, value_key::ValueKey},
    },
};

impl LocalChildPathResolver for List {
    fn resolve_child(
        &mut self,
        key: &ValueKey,
    ) -> Result<&mut ValueContainer, UpdateError> {
        if let ValueKey::Index(index) = key {
            self.try_get_mut(*index).map_err(UpdateError::access_error)
        } else {
            Err(UpdateError::access_error(AccessError::InvalidIndexKey))
        }
    }
}
