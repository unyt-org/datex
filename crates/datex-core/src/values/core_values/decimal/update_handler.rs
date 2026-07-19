use crate::{
    prelude::*,
    value_updates::update_data::IncrementUpdateData,
    values::{
        core_values::decimal::Decimal, value_container::value_key::ValueKey,
    },
};

use crate::{
    shared_values::base_shared_value_container::observers::TransceiverId,
    value_updates::{errors::UpdateError, update_handler::UpdateHandlerImpl},
};
use core::{
    ops::{AddAssign, SubAssign},
    result::Result,
};
use crate::value_updates::update_data::DecrementUpdateData;
use crate::value_updates::update_handler::UpdateCallbackData;

impl UpdateHandlerImpl for Decimal {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData> {
        None
    }

    fn try_increment(
        &mut self,
        data: IncrementUpdateData,
    ) -> Result<(), UpdateError> {
        let value = data
            .value
            .try_as::<Decimal>()
            .ok_or(UpdateError::InvalidUpdate)?;
        self.add_assign(value.clone());
        Ok(())
    }
    fn try_decrement(
        &mut self,
        data: DecrementUpdateData,
    ) -> Result<(), UpdateError> {
        let value = data
            .value
            .try_as::<Decimal>()
            .ok_or(UpdateError::InvalidUpdate)?;
        self.sub_assign(value.clone());
        Ok(())
    }
}
