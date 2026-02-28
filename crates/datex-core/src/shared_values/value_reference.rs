use crate::{
    shared_values::{observers::Observer, reference::ReferenceMutability},
    traits::value_eq::ValueEq,
    types::definition::TypeDefinition,
    utils::freemap::FreeHashMap,
    values::{
        pointer::PointerAddress, value::Value, value_container::ValueContainer,
    },
};

use crate::prelude::*;
use core::{cell::RefCell, fmt::Debug, prelude::rust_2024::*};

pub struct SharedValueContainer {
    pub pointer_address: Option<PointerAddress>,
    /// the value that this reference points to
    pub value_container: ValueContainer,
    /// pointer id, can be initialized as None for local pointers
    /// custom type for the pointer that the Datex value is allowed to reference
    pub allowed_type: TypeDefinition,
    /// list of observer callbacks
    pub observers: FreeHashMap<u32, Observer>,
    pub mutability: ReferenceMutability,
}

impl Default for SharedValueContainer {
    fn default() -> Self {
        SharedValueContainer {
            value_container: ValueContainer::Local(Value::null()),
            pointer_address: None,
            allowed_type: TypeDefinition::Unknown,
            observers: FreeHashMap::new(),
            mutability: ReferenceMutability::Immutable,
        }
    }
}

impl SharedValueContainer {
    pub fn new(
        value_container: ValueContainer,
        pointer_address: Option<PointerAddress>,
        allowed_type: TypeDefinition,
        mutability: ReferenceMutability,
    ) -> Self {
        SharedValueContainer {
            value_container,
            pointer_address,
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
            .field("pointer", &self.pointer_address)
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
    pub fn pointer_address(&self) -> &Option<PointerAddress> {
        &self.pointer_address
    }

    pub fn current_value_container(&self) -> &ValueContainer {
        &self.value_container
    }

    pub fn resolve_current_value(&self) -> Rc<RefCell<Value>> {
        self.value_container.to_value()
    }

    pub fn is_mutable(&self) -> bool {
        core::matches!(self.mutability, ReferenceMutability::Mutable)
    }
}
