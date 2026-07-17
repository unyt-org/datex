//! This module contains the implementation of modifications that can be performed on [ValueContainer]s
use crate::{
    global::protocol_structures::instruction_data::ModifySharedContainerValue,
    runtime::execution::{
        ExecutionError,
        execution_loop::implementation::handle_assignment_operation,
    },
    shared_values::{
        base_shared_value_container::observers::TransceiverId,
        traits::SharedContainerCommon,
    },
    value_updates::{
        update_data::{ReplaceUpdateData, SetEntryUpdateData, UpdateData},
        update_handler::UpdateHandler,
    },
    values::value_container::{ValueContainer, value_key::ValueKey},
};

/// Modifies the value of a shared container by applying the specified [ModifySharedContainerValue] operation.
/// If the target [ValueContainer] is not a shared container, an [ExecutionError::ExpectedSharedValue] is returned.
pub fn modify_shared_container_value(
    set_shared_container_value: ModifySharedContainerValue,
    target: &ValueContainer,
    value: ValueContainer,
    source_id: TransceiverId,
) -> Result<ValueContainer, ExecutionError> {
    if let Some(reference) = target.maybe_shared() {
        let update_data = {
            let lhs = reference.value_container();
            let val = (handle_assignment_operation(
                set_shared_container_value.operator,
                &lhs,
                value,
            ))?;
            ReplaceUpdateData { value: val }
        };
        Ok(reference
            .base_shared_container_mut()
            .try_replace(update_data, source_id)?)
    } else {
        Err(ExecutionError::ExpectedSharedValue)
    }
}

/// Sets a property on the target [ValueContainer] using the provided key and value.
/// If the property cannot be set, an [ExecutionError::UpdateError] is returned.
pub fn set_property(
    target: &mut ValueContainer,
    key: ValueKey,
    value: ValueContainer,
    transceiver_id: TransceiverId,
) -> Result<Option<ValueContainer>, ExecutionError> {
    target
        .try_set_entry(SetEntryUpdateData { key, value }, transceiver_id) // TODO #644: set correct source id
        .map_err(ExecutionError::from)
}

/// Sets the value of a shared container to the provided [ValueContainer].
/// If the target [ValueContainer] is not a shared container, an [ExecutionError::ExpectedSharedValue] is returned.
pub fn set_shared_container_value(
    target: &ValueContainer,
    new_value: ValueContainer,
    source_id: TransceiverId,
) -> Result<(), ExecutionError> {
    // TODO: check if caller endpoint can actually mutate the container

    if let Some(reference) = target.maybe_shared() {
        let update =
            UpdateData::Replace(ReplaceUpdateData { value: new_value })
                .with_source(source_id);
        Ok(reference.update(update).map(|_| ())?) // FIXME do we want to return the old value that was replaced from the execution?
    } else {
        Err(ExecutionError::ExpectedSharedValue)
    }
}
