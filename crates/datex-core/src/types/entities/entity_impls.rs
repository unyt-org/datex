use crate::values::core_values::callable::Callable;
use crate::prelude::*;

/// Definition of an entity implementation, defining the methods that can be called on an entity.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EntityImpl {
    // TODO: optional contract
    /// The methods defined in this implementation contract.
    pub methods: Vec<EntityImplMethod>,
    pub static_methods: Vec<()>, // TODO: shared or local callables
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
