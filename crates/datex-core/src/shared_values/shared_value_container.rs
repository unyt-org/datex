use crate::{
    shared_values::{
        observers::Observer, shared_container::SharedContainerMutability,
    },
    traits::value_eq::ValueEq,
    types::definition::TypeDefinition,
    utils::freemap::FreeHashMap,
    values::{value::Value, value_container::ValueContainer},
};

use crate::{prelude::*, shared_values::pointer::Pointer};
use core::{cell::RefCell, fmt::Debug, prelude::rust_2024::*};

pub struct SharedValueContainer {
    pub(crate) pointer: Pointer,
    /// the value that this reference points to
    pub value_container: ValueContainer,
    /// pointer id, can be initialized as None for local pointers
    /// custom type for the pointer that the Datex value is allowed to reference
    pub allowed_type: TypeDefinition,
    /// list of observer callbacks
    pub observers: FreeHashMap<u32, Observer>,
    pub mutability: SharedContainerMutability,
}

impl SharedValueContainer {
    pub fn new(
        value_container: ValueContainer,
        pointer: Pointer,
        allowed_type: TypeDefinition,
        mutability: SharedContainerMutability,
    ) -> Self {
        SharedValueContainer {
            value_container,
            pointer,
            allowed_type,
            observers: FreeHashMap::new(),
            mutability,
        }
    }
}

impl Debug for SharedValueContainer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReferenceData")
            .field("value_container", &self.value_container)
            .field("pointer_address", &self.pointer.address())
            .field("allowed_type", &self.allowed_type)
            .field("observers", &self.observers.len())
            .finish()
    }
}

impl PartialEq for SharedValueContainer {
    fn eq(&self, other: &Self) -> bool {
        // Two value references are equal if their value containers are equal
        self.value_container.value_eq(&other.value_container)
    }
}

impl SharedValueContainer {
    pub fn current_value_container(&self) -> &ValueContainer {
        &self.value_container
    }

    pub fn resolve_current_value(&self) -> Rc<RefCell<Value>> {
        self.value_container.to_value()
    }

    pub fn is_mutable(&self) -> bool {
        core::matches!(self.mutability, SharedContainerMutability::Mutable)
    }
    
    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }
}
