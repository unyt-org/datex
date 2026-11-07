use crate::{
    global::root_properties::RootProperty,
    runtime::{
        Runtime,
        execution::{ExecutionError, execution_input::ExecutionCallerMetadata},
    },
    values::{value::Value, value_container::ValueContainer},
};

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
        RootProperty::CONFIG => {
            ValueContainer::Local(Value::native(runtime.config().clone()))
        }
    };
    Ok(res)
}
