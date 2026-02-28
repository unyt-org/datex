use crate::{
    global::slots::InternalSlot,
    runtime::execution::{
        ExecutionError, execution_loop::state::RuntimeExecutionState,
    },
    values::{core_values::map::Map, value_container::ValueContainer},
};
use num_enum::TryFromPrimitive;

pub fn get_slot_value(
    runtime_state: &RuntimeExecutionState,
    address: u32,
) -> Result<&ValueContainer, ExecutionError> {
    runtime_state.slots.get_slot_value(address)
}

pub fn get_internal_slot_value(
    runtime_state: &RuntimeExecutionState,
    slot: u32,
) -> Result<ValueContainer, ExecutionError> {
    if let Some(runtime) = &runtime_state.runtime_internal {
        // convert slot to InternalSlot enum
        let slot = InternalSlot::try_from_primitive(slot)
            .map_err(|_| ExecutionError::SlotNotAllocated(slot))?;
        let res = match slot {
            InternalSlot::ENDPOINT => {
                ValueContainer::from(runtime.endpoint.clone())
            }
            InternalSlot::ENV => {
                ValueContainer::from(Map::from(runtime.get_env()))
            }
        };
        Ok(res)
    } else {
        Err(ExecutionError::RequiresRuntime)
    }
}
