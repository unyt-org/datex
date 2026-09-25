use crate::{
    preludes::derive::SharedReferencesCache,
    value_updates::update_data::IncrementUpdateData,
    values::core_values::decimal::Decimal,
};

use crate::value_updates::{
    errors::UpdateError,
    update_data::DecrementUpdateData,
    update_handler::{
        UpdateCallbackData, UpdateCallbackDataAccess, UpdateHandlerImpl,
    },
};
use core::{
    cell::RefCell,
    ops::{AddAssign, SubAssign},
    result::Result,
};

impl UpdateCallbackDataAccess for Decimal {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData> {
        None
    }
}

impl UpdateHandlerImpl for Decimal {
    fn try_increment(
        &mut self,
        data: IncrementUpdateData,
        _cache: &RefCell<SharedReferencesCache>,
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
        _cache: &RefCell<SharedReferencesCache>,
    ) -> Result<(), UpdateError> {
        let value = data
            .value
            .try_as::<Decimal>()
            .ok_or(UpdateError::InvalidUpdate)?;
        self.sub_assign(value.clone());
        Ok(())
    }
}
