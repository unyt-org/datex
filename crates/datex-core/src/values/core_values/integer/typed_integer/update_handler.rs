use crate::{
    value_updates::update_data::IncrementUpdateData,
    values::core_values::integer::typed_integer::TypedInteger,
};

use crate::value_updates::{
    errors::UpdateError,
    update_data::DecrementUpdateData,
    update_handler::{
        UpdateCallbackData, UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
};
use core::{
    ops::{AddAssign, SubAssign},
    result::Result,
};

impl UpdateCallbackDataAccess for TypedInteger {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData> {
        None
    }
}

impl UpdateHandlerImpl for TypedInteger {
    fn try_increment(
        &mut self,
        data: IncrementUpdateData,
    ) -> Result<(), UpdateError> {
        let value = data
            .value
            .try_as::<TypedInteger>()
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
            .try_as::<TypedInteger>()
            .ok_or(UpdateError::InvalidUpdate)?;
        self.sub_assign(value.clone());
        Ok(())
    }
}
