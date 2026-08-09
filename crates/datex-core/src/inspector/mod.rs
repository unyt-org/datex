use crate::{
    libs::core::type_id::CoreLibBaseTypeId,
    runtime::{Runtime, execution::ExecutionError},
    types::{
        r#type::Type,
        type_definition::callable::{CallableKind, CallableTypeDefinition},
    },
    values::{
        core_values::callable::{
            Callable, CallableBody, NativeCallable, error::CallableError,
        },
        value_container::ValueContainer,
    },
};
use datex_macros_internal::Datex;

#[derive(Datex, Debug)]
pub struct Inspector {}

/// Creates a new Inspector instance.
fn inspector_create(
    args: &[ValueContainer],
) -> Result<Option<ValueContainer>, CallableError> {
    Ok(Some(ValueContainer::from(Inspector {})))
}

/// Registers the `inspector` namespace in the runtime, allowing users to create Inspector instances.
pub fn register_inspector_namespace(runtime: &Runtime) {
    let inspector_create_callable = ValueContainer::from(Callable {
        name: Some("inspector".to_string()),
        signature: CallableTypeDefinition {
            kind: CallableKind::Function,
            parameter_types: vec![],
            rest_parameter_type: None,
            return_type: None,
            yeet_type: None,
        },
        body: CallableBody::Native(NativeCallable::from(inspector_create)),
        creator: Default::default(),
    });

    runtime
        .endpoint_properties_mut()
        .insert("inspector".to_string(), inspector_create_callable);
}
