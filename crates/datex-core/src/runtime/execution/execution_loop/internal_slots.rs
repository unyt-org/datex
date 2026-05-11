use crate::{
    global::{
        protocol_structures::instruction_data::StackIndex, root_properties::RootProperty,
    },
    runtime::execution::{
        ExecutionError, execution_loop::state::RuntimeExecutionState,
    },
    values::{core_values::map::Map, value_container::ValueContainer},
};
use num_enum::TryFromPrimitive;
use crate::datex_proxy::DatexValueContainerProxyInfallibleSerialize;

pub fn get_stack_value(
    runtime_state: &RuntimeExecutionState,
    index: StackIndex,
) -> Result<&ValueContainer, ExecutionError> {
    runtime_state.stack.get_stack_value(index)
}

pub fn get_root_property(
    runtime_state: &RuntimeExecutionState,
    root_property: RootProperty,
) -> Result<ValueContainer, ExecutionError> {
    let runtime = &runtime_state.runtime;
    let res = match root_property {
        RootProperty::ENDPOINT => {
            ValueContainer::from(runtime.endpoint().clone())
        }
        RootProperty::CALLER => {
            ValueContainer::from(runtime_state.caller_metadata.endpoint.clone())
        }
        RootProperty::ENV => {
            ValueContainer::from(Map::from(runtime.internal.get_env()))
        }
        RootProperty::CONFIG => {
            runtime_state.runtime.config().clone().to_value_container()
        }
    };
    Ok(res)
}
