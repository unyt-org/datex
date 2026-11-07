use crate::{
    global::root_properties::RootProperty,
    runtime::execution::{
        ExecutionError, execution_loop::state::RuntimeExecutionState,
    },
    values::{value::Value, value_container::ValueContainer},
};
use crate::runtime::execution::execution_input::ExecutionCallerMetadata;
use crate::runtime::Runtime;

pub fn get_root_property(
    runtime: &Runtime,
    caller_metadata: &ExecutionCallerMetadata,
    root_property: RootProperty,
) -> Result<ValueContainer, ExecutionError> {
    let res = match root_property {
        RootProperty::ENDPOINT => {
            ValueContainer::from(runtime.endpoint().clone())
        }
        RootProperty::CALLER => {
            ValueContainer::from(caller_metadata.endpoint.clone())
        }
        RootProperty::ENV => ValueContainer::from(runtime.internal.get_env()),
        RootProperty::NAME => ValueContainer::from(runtime.internal.get_name()),
        RootProperty::CONFIG => ValueContainer::Local(Value::native(
            runtime.config().clone(),
        )),
    };
    Ok(res)
}
