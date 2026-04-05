use crate::{
    global::slots::InternalSlot,
    runtime::execution::{
        ExecutionError, execution_loop::state::RuntimeExecutionState,
    },
    values::{core_values::map::Map, value_container::ValueContainer},
};
use num_enum::TryFromPrimitive;
use crate::global::protocol_structures::instruction_data::StackIndex;

pub fn get_stack_value(
    runtime_state: &RuntimeExecutionState,
    index: StackIndex,
) -> Result<&ValueContainer, ExecutionError> {
    runtime_state.stack.get_stack_value(index)
}

pub fn get_internal_stack_value(
    runtime_state: &RuntimeExecutionState,
    slot: u32,
) -> Result<ValueContainer, ExecutionError> {
    let runtime = &runtime_state.runtime_internal;
    // convert slot to InternalSlot enum
    let slot = InternalSlot::try_from_primitive(slot)
        .map_err(|_| ExecutionError::InternalSlotDoesNotExist(slot))?;
    let res = match slot {
        InternalSlot::ENDPOINT => {
            ValueContainer::from(runtime.endpoint.clone())
        }
        InternalSlot::CALLER => ValueContainer::from(runtime_state.caller_metadata.endpoint.clone()),
        InternalSlot::ENV => ValueContainer::from(Map::from(runtime.get_env())),
    };
    Ok(res)
}
