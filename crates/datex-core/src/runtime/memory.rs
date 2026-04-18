use crate::{
    collections::HashMap,
    prelude::*,
    shared_values::{
        pointer_address::{
            PointerAddress,
        },
        shared_containers::{
            ReferencedSharedContainer,
        },
    },
    values::core_values::endpoint::Endpoint,
};
use crate::libs::core::CoreLibrary;
use crate::libs::library::Library;

#[derive(Debug)]
pub struct Memory {
    /// Shared values that are actively referenced or owned somewhere
    /// in the runtime or on remote endpoints
    pointers: HashMap<PointerAddress, ReferencedSharedContainer>,
}

impl Memory {
    /// Creates a new, Memory instance with the core library loaded.
    pub fn new() -> Memory {
        let mut memory = Memory {
            pointers: HashMap::new(),
        };
        // load core library
        // Note: safe because memory is newly initialized without core lib
        unsafe {
            CoreLibrary::load(&mut memory);
        }

        memory
    }

    /// Registers a referenced shared container in memory. If the reference has no PointerAddress, a new local one is generated.
    /// If the reference is already registered (has a PointerAddress), the existing address is returned and no new registration is done.
    /// Owned shared containers shall not be registered in memory.
    /// Returns the PointerAddress of the registered reference.
    pub fn register_referenced_shared_container(
        &mut self,
        container: &ReferencedSharedContainer,
    ) {
        let pointer_address = container.pointer_address();
        // check if reference is already registered (if it has an address, we assume it is registered)
        self.pointers
            .entry(pointer_address)
            .or_insert_with(|| container.clone());
    }

    /// Returns a reference stored at the given PointerAddress, if it exists.
    pub fn get_reference(
        &self,
        pointer_address: &PointerAddress,
    ) -> Option<&ReferencedSharedContainer> {
        self.pointers.get(pointer_address)
    }

    /// Checks if a reference with the given PointerAddress exists in memory.
    pub fn has_reference(&self, pointer_address: &PointerAddress) -> bool {
        self.pointers.contains_key(pointer_address)
    }
}