use crate::{
    global::protocol_structures::instruction_data::ModifySharedContainerValue,
    runtime::execution::{
        ExecutionError, execution_loop::operations::handle_assignment_operation,
    },
    shared_values::{
        base_shared_value_container::observers::TransceiverId,
        traits::SharedContainerCommon,
    },
    value_updates::{
        errors::UpdateError, update_data::ReplaceUpdateData,
        update_handler::UpdateHandler,
    },
    values::value_container::ValueContainer,
};

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
