use crate::{
    prelude::*,
    value_updates::update_data::IncrementUpdateData,
    values::{
        core_values::integer::{Integer, typed_integer::TypedInteger},
        value_container::value_key::ValueKey,
    },
};

use crate::{
    shared_values::base_shared_value_container::observers::TransceiverId,
    value_updates::{errors::UpdateError, update_handler::UpdateHandlerImpl},
};
use core::{
    ops::{Add, AddAssign, SubAssign},
    result::Result,
};

impl UpdateHandlerImpl for TypedInteger {
    fn try_increment(
        &mut self,
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
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
        _path: Vec<ValueKey>,
        _source_id: TransceiverId,
        data: crate::value_updates::update_data::DecrementUpdateData,
    ) -> Result<(), UpdateError> {
        let value = data
            .value
            .try_as::<TypedInteger>()
            .ok_or(UpdateError::InvalidUpdate)?;
        self.sub_assign(value.clone());
        Ok(())
    }
}
