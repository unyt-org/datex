//! This module contains the implementation of modifications that can be performed on [ValueContainer]s
use crate::{
    global::operators::ModificationOperator,
    prelude::*,
    runtime::execution::ExecutionError,
    shared_values::base_shared_value_container::observers::TransceiverId,
    types::traits::operator_handler::OperatorHandler,
    value_updates::{
        UpdateReturn,
        errors::UpdateError,
        update_data::{
            AppendEntryUpdateData, DecrementUpdateData, DeleteEntryUpdateData,
            ReplaceUpdateData, SetEntryUpdateData, Update, UpdateData,
            UpdateModificationOperator, UpdateOperation,
        },
        update_handler::UpdateHandler,
    },
    values::value_container::{ValueContainer, value_key::ValueKey},
};
use core::assert_matches;
/// Modifies the value of a value container by applying the specified [ModificationOperator] operation.
/// If the target [ValueContainer] is not a shared container, an [ExecutionError::ExpectedSharedValue] is returned.
pub fn try_modify_value_container(
    target: &mut ValueContainer,
    operator: ModificationOperator,
    value: ValueContainer,
    source_id: TransceiverId,
    path: Vec<ValueKey>,
) -> Result<(), ExecutionError> {
    let res = target.try_handle_update(Update::new(
        source_id,
        UpdateData::new_with_path(
            try_create_update_operation_for_modification(
                operator, &target, value,
            )?,
            path,
        ),
    ))?;
    // Due to the way modifications work, we expect to do a inline mutation and can not return
    // any value (append_entry, ...)
    assert_matches!(res, UpdateReturn::None);
    Ok(())
}

fn try_create_update_operation_for_modification(
    operator: ModificationOperator,
    target: &ValueContainer,
    value: ValueContainer,
) -> Result<UpdateOperation, ExecutionError> {
    match target
        .actual_type()
        .get_update_type_for_modification(operator)
        .map_err(|_| ExecutionError::update_error(UpdateError::InvalidUpdate))?
    {
        UpdateModificationOperator::AppendEntry => {
            Ok(UpdateOperation::append_entry(value))
        }
        UpdateModificationOperator::Increment => {
            Ok(UpdateOperation::replace(value))
        }
        UpdateModificationOperator::Decrement => {
            Ok(UpdateOperation::decrement(value))
        }
        UpdateModificationOperator::DeleteEntry => {
            Ok(UpdateOperation::delete_entry(
                ValueKey::from(value), // FIXME
            ))
        }
    }
}

/// Sets a property on the target [ValueContainer] using the provided key and value.
/// If the property cannot be set, an [ExecutionError::UpdateError] is returned.
pub fn try_set_property(
    target: &mut ValueContainer,
    key: ValueKey,
    value: ValueContainer,
    path: Vec<ValueKey>,
    transceiver_id: TransceiverId,
) -> Result<Option<ValueContainer>, ExecutionError> {
    target
        .try_set_entry(
            path,
            transceiver_id,
            SetEntryUpdateData::new(key, value),
        )
        .map_err(ExecutionError::from)
}

/// Sets the value of a shared container to the provided [ValueContainer].
/// If the target [ValueContainer] is not a shared container, an [ExecutionError::ExpectedSharedValue] is returned.
pub fn set_shared_container_value(
    target: &mut ValueContainer,
    new_value: ValueContainer,
    source_id: TransceiverId,
) -> Result<(), ExecutionError> {
    // TODO: check if caller endpoint can actually mutate the container
    if let Some(reference) = target.maybe_shared_mut() {
        let update =
            UpdateOperation::replace(new_value).with_source_root(source_id); // here we defintely want to update the container, we do not have a path to a value update
        Ok(reference.try_handle_update(update).map(|_| ())?) // FIXME do we want to return the old value that was replaced from the execution?
    } else {
        Err(ExecutionError::ExpectedSharedValue)
    }
}
