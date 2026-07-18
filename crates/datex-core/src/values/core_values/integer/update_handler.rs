use crate::{
    prelude::*,
    value_updates::update_data::IncrementUpdateData,
    values::{
        core_values::integer::Integer, value_container::value_key::ValueKey,
    },
};

use crate::{
    shared_values::base_shared_value_container::observers::TransceiverId,
    value_updates::{errors::UpdateError, update_handler::UpdateHandlerImpl},
};
use core::result::Result;

impl UpdateHandlerImpl for Integer {
    fn try_increment(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
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
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        data: crate::value_updates::update_data::DecrementUpdateData,
    ) -> Result<(), UpdateError> {
        let value = data
            .value
            .try_as::<Integer>()
            .ok_or(UpdateError::InvalidUpdate)?;
        self.0 -= &value.0;
        Ok(())
    }
}
