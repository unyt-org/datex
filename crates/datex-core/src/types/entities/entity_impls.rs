use crate::{
    prelude::*, types::type_definition::callable::CallableTypeDefinition,
    values::core_values::callable::Callable,
};

/// Definition of an entity implementation, defining the methods that can be called on an entity.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EntityImpl {
    // TODO: optional contract
    /// The methods defined in this implementation contract.
    pub methods: Vec<EntityImplMethod>,
    pub static_methods: Vec<Callable>, // TODO: shared or local callables
}

impl EntityImpl {
    pub fn new(
        methods: Vec<EntityImplMethod>,
        static_methods: Vec<Callable>,
    ) -> Self {
        Self {
            methods,
            static_methods,
        }
    }

    /// Returns a reference to the method for the given method name, if it exists in this implementation.
    pub fn try_get_method(
        &self,
        method_name: &str,
    ) -> Option<&EntityImplMethod> {
        self.methods.iter().find(|method| {
            method
                .name()
                .map(|name| name == method_name)
                .unwrap_or(false)
        })
    }

    /// Returns a reference to the (static) method for the given property name, if it exists in this implementation.
    pub fn try_get_property(&self, property_name: &str) -> Option<&Callable> {
        self.methods
            .iter()
            .map(|met| &met.callable)
            .chain(self.static_methods.iter())
            .find(|method| {
                method
                    .name
                    .as_ref()
                    .map(|name| name == property_name)
                    .unwrap_or(false)
            })
    }
}

/// Represents a method in an entity implementation.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EntityImplMethod {
    /// When true, the method should is called on the owner endpoint of the value for which the method is invoked.
    /// When false, the method is called locally on the current caller endpoint.
    pub call_on_owner: bool,
    /// The callable that implements the method.
    pub callable: Callable,
}
impl EntityImplMethod {
    pub fn new(call_on_owner: bool, callable: Callable) -> Self {
        Self {
            call_on_owner,
            callable,
        }
    }
    pub fn signature(&self) -> &CallableTypeDefinition {
        &self.callable.signature
    }
    pub fn name(&self) -> Option<&String> {
        self.callable.name.as_ref()
    }
}
