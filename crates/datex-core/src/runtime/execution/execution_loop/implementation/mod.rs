use crate::{
    global::protocol_structures::instruction_data::ModifySharedContainerValue,
    runtime::execution::ExecutionError,
    shared_values::{
        base_shared_value_container::observers::TransceiverId,
        traits::SharedContainerCommon,
    },
    types::{
        r#type::Type,
        type_definition::{TypeDefinition, tagged_type::TaggedTypeDefinition},
    },
    value_updates::{
        errors::UpdateError, update_data::ReplaceUpdateData,
        update_handler::UpdateHandler,
    },
    values::value_container::ValueContainer,
};

mod operations;
pub use operations::*;

mod create;
pub use create::*;

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
