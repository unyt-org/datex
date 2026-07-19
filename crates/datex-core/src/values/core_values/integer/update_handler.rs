use crate::{
    prelude::*,
    value_updates::update_data::IncrementUpdateData,
    values::{
        core_values::integer::Integer, value_container::value_key::ValueKey,
    },
};

use crate::{
    shared_values::base_shared_value_container::observers::TransceiverId,
    value_updates::{
        errors::UpdateError,
        update_data::DecrementUpdateData,
        update_handler::{UpdateCallbackData, UpdateHandlerImpl},
    },
};
use core::result::Result;
use crate::value_updates::update_handler::UpdateCallbackDataAccess;

impl UpdateCallbackDataAccess for Integer {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData> {
        None
    }
}

impl UpdateHandlerImpl for Integer {

    fn try_increment(
        &mut self,
        data: IncrementUpdateData,
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
    ) -> Result<(), UpdateError> {
        let value = data
            .value
            .try_as::<Integer>()
            .ok_or(UpdateError::InvalidUpdate)?;
        self.0 -= &value.0;
        Ok(())
    }
}
