use core::cell::RefCell;

use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    shared_values::base_shared_value_container::observers::TransceiverId,
    value_updates::{
        errors::UpdateError,
        update_data::UpdateOperation,
        update_handler::{
            InternalMutabilityUpdateHandler, UpdateCallbackData,
            UpdateCallbackDataAccess, UpdateHandler, UpdateHandlerImpl,
            UpdateResult,
        },
    },
    values::{
        core_value::CoreValue,
        core_value_with_classification::CoreValueWithClassification,
    },
};

impl InternalMutabilityUpdateHandler for CoreValueWithClassification {
    fn set_update_callback_data(
        &mut self,
        observe_data: Option<UpdateCallbackData>,
    ) {
        match &mut self.inner {
            CoreValue::Map(map) => map.set_update_callback_data(observe_data),
            CoreValue::List(list) => {
                list.set_update_callback_data(observe_data)
            }
            _ => {}
        }
    }
}

impl UpdateCallbackDataAccess for CoreValueWithClassification {
    fn get_update_callback_data(&self) -> Option<&UpdateCallbackData> {
        match &self.inner {
            CoreValue::Map(map) => map.get_update_callback_data(),
            CoreValue::List(list) => list.get_update_callback_data(),
            _ => None,
        }
    }
}

impl UpdateHandlerImpl for CoreValueWithClassification {
    /// Tries to update the value with the given operation.
    /// If a path first needs to be resolved, use [Value::try_update_collapsed_local_inner]
    fn try_update(
        &mut self,
        operation: UpdateOperation,
        source_id: Option<TransceiverId>,
        cache: &RefCell<SharedReferencesCache>,
    ) -> UpdateResult {
        match &mut self.inner {
            // collections
            CoreValue::Map(map) => map.try_update(operation, source_id, cache),
            CoreValue::List(list) => {
                list.try_update(operation, source_id, cache)
            }
            CoreValue::Integer(integer) => {
                integer.try_update(operation, source_id, cache)
            }
            CoreValue::Decimal(decimal) => {
                decimal.try_update(operation, source_id, cache)
            }
            CoreValue::TypedInteger(integer) => {
                integer.try_update(operation, source_id, cache)
            }
            CoreValue::TypedDecimal(decimal) => {
                decimal.try_update(operation, source_id, cache)
            }
            _ => Err(UpdateError::InvalidUpdate),
        }
    }
}
