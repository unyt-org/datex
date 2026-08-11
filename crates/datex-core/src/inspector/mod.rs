use crate::{
    datex_proxy::DatexProxyTypes, runtime::Runtime,
    values::value_container::ValueContainer,
};
use core::ops::DerefMut;
use datex_macros_internal::{Datex, datex};

#[derive(Datex, Debug)]
pub struct Inspector {}

#[datex]
impl Inspector {
    /// Creates a new Inspector instance.
    pub fn create() -> Self {
        Inspector {}
    }
}

/// Registers the `inspector` namespace in the runtime, allowing users to create Inspector instances.
pub fn register_inspector_namespace(runtime: &Runtime) {
    let mut memory = runtime.memory().borrow_mut();
    let _inspector =
        ValueContainer::from(Inspector::datex_type(memory.deref_mut()));
    // let inspector_create_callable = ValueContainer::from(Callable {
    //     name: Some("inspector".to_string()),
    //     signature: CallableTypeDefinition {
    //         kind: CallableKind::Function,
    //         parameter_types: vec![],
    //         rest_parameter_type: None,
    //         return_type: None,
    //         yeet_type: None,
    //     },
    //     body: CallableBody::Native(NativeCallable::from(inspector_create)),
    //     creator: Default::default(),
    // });
    //
    // runtime
    //     .endpoint_properties_mut()
    //     .insert("inspector".to_string(), inspector_create_callable);
}
