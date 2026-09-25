use crate::{
    preludes::derive::SharedReferencesCache,
    value_updates::update_data::IncrementUpdateData,
    values::core_values::integer::Integer,
};

use crate::value_updates::{
    errors::UpdateError,
    update_data::DecrementUpdateData,
    update_handler::{
        UpdateCallbackData, UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
};
use core::{cell::RefCell, result::Result};

impl UpdateCallbackDataAccess for Integer {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData> {
        None
    }
}

impl UpdateHandlerImpl for Integer {
    fn try_increment(
        &mut self,
        data: IncrementUpdateData,
        _cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        let value = data
            .value
            .try_as::<Integer>()
            .ok_or(UpdateError::InvalidUpdate)?;
        self.0 += &value.0;
        Ok(())
    }
    fn try_decrement(
        &mut self,
        data: DecrementUpdateData,
        _cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        let value = data
            .value
            .try_as::<Integer>()
            .ok_or(UpdateError::InvalidUpdate)?;
        self.0 -= &value.0;
        Ok(())
    }
}
